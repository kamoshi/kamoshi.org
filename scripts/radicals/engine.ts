import * as d3 from 'd3';

// --- TYPES ---
export type RelationType = 'sibling' | 'parent' | 'child';

export interface Node extends d3.SimulationNodeDatum {
  id: string;
  radicals: string[];
  status: 'visible' | 'shadow';
  type: 'root' | 'standard';
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
}

export interface Link extends d3.SimulationLinkDatum<Node> {
  source: string | Node;
  target: string | Node;
  relation: RelationType;
}

// --- DATA & HELPERS ---
const KANJI_DATA: Record<string, string[]> = {
  亜: ['｜', '一', '口'],
  唖: ['｜', '一', '口'],
  娃: ['女', '土'],
  阿: ['一', '口', '亅', '阡'],
  哀: ['衣', '口', '亠'],
  愛: ['心', '爪', '冖', '夂'],
  挨: ['矢', '厶', '扎', '乞'],
  姶: ['一', '口', '女', '个'],
  逢: ['｜', '込', '二', '夂'],
  葵: ['人', '大', '二', '癶', '艾', 'ノ'],
  茜: ['西', '艾'],
  穐: ['禾', '亀', '乙', '勹', '田'],
  悪: ['｜', '一', '口', '心'],
  握: ['至', '土', '厶', '尸', '扎'],
  渥: ['至', '汁', '土', '厶', '尸'],
  旭: ['日', '九'],
  葦: ['口', '艾', '韋'],
  芦: ['戸', '艾', '一', '尸'],
  鯵: ['魚', '大', '田', '厶', '彡', '杰'],
  梓: ['十', '辛', '木', '立'],
  圧: ['土', '厂'],
  斡: ['十', '斗', '日', '个'],
  扱: ['扎', '及'],
  宛: ['夕', '卩', '宀'],
  姐: ['女', '目'],
  虻: ['虫', '亡', '亠'],
  飴: ['口', '食', '厶'],
  絢: ['糸', '幺', '小', '日', '勹'],
  綾: ['糸', '幺', '小', '土', '儿', '夂'],
  鮎: ['魚', '口', '田', '卜', '杰'],
  或: ['口', '戈', '一'],
  粟: ['西', '米'],
  袷: ['口', '初', '个', '一'],
  安: ['女', '宀'],
  庵: ['田', '广', '大'],
  按: ['女', '宀', '扎'],
  暗: ['音', '日', '立'],
  案: ['女', '木', '宀'],
  闇: ['音', '日', '門', '立'],
  鞍: ['女', '宀', '革'],
  杏: ['口', '木'],
  以: ['｜', '人', '丶'],
  伊: ['｜', 'ヨ', '化'],
  位: ['化', '立'],
  依: ['衣', '化', '亠'],
  偉: ['化', '口', '韋'],
  囲: ['囗', '井'],
  夷: ['ノ', '一', '弓', '大'],
  口: ['口'],
  心: ['心'],
  一: ['一'],
  '｜': ['｜'],
  女: ['女'],
  木: ['木'],
  日: ['日'],
  土: ['土'],
};

const isSubset = (subset: string[], superset: string[]) => {
  const setB = new Set(superset);
  return subset.every((val) => setB.has(val));
};

const isEqualSet = (arr1: string[], arr2: string[]) => {
  if (arr1.length !== arr2.length) return false;
  const set1 = new Set(arr1);
  return arr2.every((val) => set1.has(val));
};

const determineRelationship = (
  radicalsA: string[],
  radicalsB: string[],
): RelationType | null => {
  if (isEqualSet(radicalsA, radicalsB)) return 'sibling';
  if (isSubset(radicalsA, radicalsB)) return 'child';
  if (isSubset(radicalsB, radicalsA)) return 'parent';
  return null;
};

// --- THE ENGINE ---

export class KanjiGraphEngine {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private width: number = 0;
  private height: number = 0;

  // Physics
  private simulation: d3.Simulation<Node, Link>;
  private nodes: Node[] = [];
  private links: Link[] = [];
  private transform = d3.zoomIdentity;

  // Interaction
  public onNodeSelect: ((node: Node | null) => void) | null = null;
  private selectedNodeId: string | null = null;

  constructor(canvas: HTMLCanvasElement) {
    const ctx = canvas.getContext('2d');
    if (!canvas || !ctx) {
      throw new Error('Canvas context not available');
    }

    this.canvas = canvas;
    this.ctx = ctx;

    // 1. Init Simulation FIRST
    this.simulation = d3
      .forceSimulation<Node, Link>()
      .force(
        'link',
        d3
          .forceLink<Node, Link>()
          .id((d: any) => d.id)
          .distance(100),
      )
      .force('charge', d3.forceManyBody().strength(-300))
      // Center force will be updated in resize,
      // but we need a placeholder or initial value here to prevent errors if we didn't use resize immediately.
      .force('center', d3.forceCenter(this.width / 2, this.height / 2))
      .force('collide', d3.forceCollide().radius(30));

    this.simulation.on('tick', () => this.render());

    this.setupInteraction();

    // 2. Initial Resize SECOND (Now this.simulation exists)
    this.resize();
  }

  public resize() {
    const parent = this.canvas.parentElement;
    if (parent) {
      const rect = parent.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;

      this.canvas.width = rect.width * dpr;
      this.canvas.height = rect.height * dpr;

      // CSS display size
      this.canvas.style.width = `${rect.width}px`;
      this.canvas.style.height = `${rect.height}px`;

      this.width = this.canvas.width;
      this.height = this.canvas.height;

      // --- SAFETY CHECK ---
      // If simulation hasn't been created yet, stop here.
      if (!this.simulation) return;

      // Update center force
      this.simulation.force(
        'center',
        d3.forceCenter(this.width / 2 / dpr, this.height / 2 / dpr),
      );
      this.simulation.alpha(0.3).restart();
    }
  }

  public destroy() {
    this.simulation.stop();
  }

  // --- LOGIC: GRAPH MANIPULATION ---

  public search(kanji: string) {
    if (KANJI_DATA[kanji]) {
      this.expandNode(kanji);
      this.selectedNodeId = kanji;
      const node = this.nodes.find((n) => n.id === kanji);
      if (this.onNodeSelect && node) this.onNodeSelect(node);
    } else {
      alert('Kanji not found in dataset');
    }
  }

  public removeNode(id: string) {
    // Downgrade to shadow
    const node = this.nodes.find((n) => n.id === id);
    if (node) node.status = 'shadow';

    // Prune logic
    const activeIds = new Set<string>();
    this.nodes.forEach((n) => {
      if (n.status === 'visible') activeIds.add(n.id);
    });

    // Filter links based on visibility
    this.links = this.links.filter((l) => {
      const sourceId = (l.source as Node).id;
      const targetId = (l.target as Node).id;
      // Keep link if either end is visible
      const sourceVisible =
        this.nodes.find((n) => n.id === sourceId)?.status === 'visible';
      const targetVisible =
        this.nodes.find((n) => n.id === targetId)?.status === 'visible';

      if (sourceVisible || targetVisible) {
        activeIds.add(sourceId);
        activeIds.add(targetId);
        return true;
      }
      return false;
    });

    // Remove orphaned shadows
    this.nodes = this.nodes.filter((n) => activeIds.has(n.id));

    this.updateSimulation();
    this.selectedNodeId = null;
  }

  public expandNode(targetId: string) {
    const targetRadicals = KANJI_DATA[targetId];
    if (!targetRadicals) return;

    // 1. Add Target
    let targetNode = this.nodes.find((n) => n.id === targetId);
    if (!targetNode) {
      targetNode = {
        id: targetId,
        radicals: targetRadicals,
        status: 'visible',
        type: this.nodes.length === 0 ? 'root' : 'standard',
        x: this.width / 2 / (window.devicePixelRatio || 1),
        y: this.height / 2 / (window.devicePixelRatio || 1),
      };
      this.nodes.push(targetNode);
    } else {
      targetNode.status = 'visible';
    }

    // 2. Add Neighbors
    Object.keys(KANJI_DATA).forEach((key) => {
      if (key === targetId) return;
      const otherRadicals = KANJI_DATA[key];
      const relation = determineRelationship(targetRadicals, otherRadicals);

      if (relation) {
        let neighbor = this.nodes.find((n) => n.id === key);
        if (!neighbor) {
          neighbor = {
            id: key,
            radicals: otherRadicals,
            status: 'shadow',
            type: 'standard',
            x: targetNode!.x,
            y: targetNode!.y,
          };
          this.nodes.push(neighbor);
        }

        const linkExists = this.links.some((l) => {
          const s = (l.source as Node).id || l.source;
          const t = (l.target as Node).id || l.target;
          return (s === targetId && t === key) || (s === key && t === targetId);
        });

        if (!linkExists) {
          if (relation === 'child')
            this.links.push({ source: targetId, target: key, relation });
          else if (relation === 'parent')
            this.links.push({
              source: key,
              target: targetId,
              relation: 'child',
            });
          else this.links.push({ source: targetId, target: key, relation });
        }
      }
    });

    this.updateSimulation();
  }

  private updateSimulation() {
    this.simulation.nodes(this.nodes);
    (this.simulation.force('link') as d3.ForceLink<Node, Link>).links(
      this.links,
    );
    this.simulation.alpha(1).restart();
  }

  // --- RENDER ---

  private render() {
    const dpr = window.devicePixelRatio || 1;

    this.ctx.save();
    this.ctx.clearRect(0, 0, this.width, this.height);

    // Transform
    this.ctx.scale(dpr, dpr);
    this.ctx.translate(this.transform.x, this.transform.y);
    this.ctx.scale(this.transform.k, this.transform.k);

    // Draw Links
    this.links.forEach((link) => {
      const source = link.source as Node;
      const target = link.target as Node;
      if (source.x === undefined || target.x === undefined) return;

      this.ctx.beginPath();
      this.ctx.moveTo(source.x, source.y);
      this.ctx.lineTo(target.x, target.y);
      this.ctx.lineWidth = link.relation === 'sibling' ? 1 : 2;
      this.ctx.strokeStyle =
        link.relation === 'sibling' ? '#94a3b8' : '#64748b';
      if (link.relation === 'sibling') this.ctx.setLineDash([4, 4]);
      else this.ctx.setLineDash([]);
      this.ctx.stroke();
    });

    // Draw Nodes
    this.nodes.forEach((node) => {
      if (node.x === undefined) return;

      const isSelected = this.selectedNodeId === node.id;

      this.ctx.beginPath();
      this.ctx.arc(node.x, node.y, 20, 0, 2 * Math.PI);

      // Fill
      if (isSelected) this.ctx.fillStyle = '#eff6ff';
      else if (node.status === 'shadow') this.ctx.fillStyle = '#f1f5f9';
      else if (node.type === 'root') this.ctx.fillStyle = '#fee2e2';
      else this.ctx.fillStyle = '#fff';
      this.ctx.fill();

      // Stroke
      this.ctx.lineWidth = isSelected ? 3 : node.status === 'shadow' ? 1 : 2;
      this.ctx.strokeStyle = isSelected
        ? '#2563eb'
        : node.type === 'root'
          ? '#ef4444'
          : '#3b82f6';
      if (node.status === 'shadow') this.ctx.setLineDash([5, 5]);
      else this.ctx.setLineDash([]);
      this.ctx.stroke();

      // Text
      this.ctx.fillStyle = node.status === 'shadow' ? '#94a3b8' : '#1e293b';
      this.ctx.font = '14px sans-serif';
      this.ctx.textAlign = 'center';
      this.ctx.textBaseline = 'middle';
      this.ctx.fillText(node.id, node.x, node.y);
    });

    this.ctx.restore();
  }

  // --- INTERACTION ---

  private setupInteraction() {
    const zoom = d3
      .zoom<HTMLCanvasElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (e) => {
        this.transform = e.transform;
        this.render(); // Force render on zoom
      });

    d3.select(this.canvas).call(zoom).on('dblclick.zoom', null); // Disable double click zoom

    // Drag
    const dragSubject = (event: any) => {
      const transform = d3.zoomTransform(this.canvas);
      const x = transform.invertX(event.x);
      const y = transform.invertY(event.y);

      let subject = null;
      let minInfo = 25;
      for (const n of this.nodes) {
        if (!n.x || !n.y) continue;
        const dist = Math.hypot(x - n.x, y - n.y);
        if (dist < minInfo) {
          minInfo = dist;
          subject = n;
        }
      }
      return subject;
    };

    const drag = d3
      .drag<HTMLCanvasElement, Node>()
      .subject(dragSubject)
      .on('start', (event) => {
        if (!event.active) this.simulation.alphaTarget(0.3).restart();
        event.subject.fx = event.subject.x;
        event.subject.fy = event.subject.y;
      })
      .on('drag', (event) => {
        const transform = d3.zoomTransform(this.canvas);
        event.subject.fx = transform.invertX(event.x);
        event.subject.fy = transform.invertY(event.y);
      })
      .on('end', (event) => {
        if (!event.active) this.simulation.alphaTarget(0);
        event.subject.fx = null;
        event.subject.fy = null;
      });

    d3.select(this.canvas).call(drag);

    // Click (Selection)
    d3.select(this.canvas).on('click', (event) => {
      // Prevent click if it was a drag
      if (event.defaultPrevented) return;

      const subject = dragSubject({ x: event.offsetX, y: event.offsetY }); // Simplified pointer
      if (subject) {
        this.selectedNodeId = subject.id;

        // Auto expand shadow nodes
        if (subject.status === 'shadow') {
          this.expandNode(subject.id);
        }

        if (this.onNodeSelect) this.onNodeSelect(subject);
      } else {
        this.selectedNodeId = null;
        if (this.onNodeSelect) this.onNodeSelect(null);
      }
      this.render();
    });
  }
}
