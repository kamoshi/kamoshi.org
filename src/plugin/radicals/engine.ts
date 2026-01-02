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

  // Data
  private data: Record<string, string[]>;

  // Physics
  private simulation: d3.Simulation<Node, Link>;
  private nodes: Node[] = [];
  private links: Link[] = [];
  private transform = d3.zoomIdentity;

  // Interaction
  public onNodeSelect: ((node: Node | null) => void) | null = null;
  private selectedNodeId: string | null = null;

  constructor(canvas: HTMLCanvasElement, data: Record<string, string[]>) {
    this.data = data ?? KANJI_DATA;

    const ctx = canvas.getContext('2d');
    if (!canvas || !ctx) {
      throw new Error('Canvas context not available');
    }

    this.canvas = canvas;
    this.ctx = ctx;

    // 1. Init Simulation FIRST
    this.simulation = d3
      .forceSimulation<Node, Link>()
      .velocityDecay(0.6)
      .force(
        'link',
        d3
          .forceLink<Node, Link>()
          .id((d: any) => d.id)
          .distance(60),
      )
      .force('charge', d3.forceManyBody().strength(-600))
      // Center force will be updated in resize,
      // but we need a placeholder or initial value here to prevent errors if we didn't use resize immediately.
      .force('center', d3.forceCenter(this.width / 2, this.height / 2))
      .force('collide', d3.forceCollide().radius(30).iterations(2))
      // Gently pulls individual nodes inward.
      .force('x', d3.forceX(this.width / 2).strength(0.1))
      .force('y', d3.forceY(this.height / 2).strength(0.1));

    this.simulation.on('tick', () => this.render());

    this.setupInteraction();

    // 2. Initial Resize SECOND (Now this.simulation exists)
    this.resize();
  }

  public resize() {
    // We measure the PARENT, not the canvas itself,
    // because the canvas is now absolute and might not have caught up yet.
    const parent = this.canvas.parentElement;

    if (parent) {
      // 1. Get the exact size of the container in CSS pixels
      const rect = parent.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;

      // 2. Set the internal resolution (physical pixels) for sharpness
      this.canvas.width = rect.width * dpr;
      this.canvas.height = rect.height * dpr;

      // 3. Set the CSS display size to match container exactly
      this.canvas.style.width = `${rect.width}px`;
      this.canvas.style.height = `${rect.height}px`;

      // 4. Update internal state dimensions (use logical/CSS pixels for D3 calculations)
      this.width = rect.width;
      this.height = rect.height;

      // 5. Update Center Force
      // Since we scale the context by DPR in render(),
      // the simulation coordinates should remain in CSS/Logical pixels.
      if (this.simulation) {
        this.simulation.force(
          'center',
          d3.forceCenter(this.width / 2, this.height / 2),
        );
        this.simulation.alpha(0.3).restart();
      }

      // 6. Force a redraw immediately
      this.render();
    }
  }

  public destroy() {
    this.simulation.stop();
  }

  // --- LOGIC: GRAPH MANIPULATION ---

  public search(kanji: string) {
    if (this.data[kanji]) {
      this.expandNode(kanji);
      this.selectedNodeId = kanji;
      const node = this.nodes.find((n) => n.id === kanji);
      if (this.onNodeSelect && node) this.onNodeSelect(node);
    } else {
      alert('Kanji not found in dataset');
    }
  }

  public promoteToRoot(targetId: string) {
    // 1. Validation: Does it exist in our dictionary?
    if (!this.data[targetId]) {
      alert('Kanji not found in dataset');
      return;
    }

    // 2. Demote existing roots
    this.nodes.forEach((n) => {
      if (n.type === 'root') {
        n.type = 'standard';
      }
    });

    // 3. Find or Create the new Root
    let targetNode = this.nodes.find((n) => n.id === targetId);

    if (targetNode) {
      // If it's already on screen (visible or shadow), just update it
      targetNode.type = 'root';
      targetNode.status = 'visible'; // Ensure it's fully visible
    } else {
      // If it's brand new to the canvas, create it
      // We spawn it near the center or near a random existing node to avoid overlap
      targetNode = {
        id: targetId,
        radicals: this.data[targetId],
        status: 'visible',
        type: 'root', // <--- Make it RED
        x: this.width / 2 / (window.devicePixelRatio || 1),
        y: this.height / 2 / (window.devicePixelRatio || 1),
      };
      this.nodes.push(targetNode);
    }

    // 4. Select it (optional, but good UX)
    this.selectedNodeId = targetId;
    if (this.onNodeSelect) this.onNodeSelect(targetNode);

    // 5. Expand its neighbors (using your existing logic)
    // This brings in the surrounding context for the new root
    this.expandNode(targetId);

    // 6. Restart Physics
    this.updateSimulation();
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
    const targetRadicals = this.data[targetId];
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
    Object.keys(this.data).forEach((key) => {
      if (key === targetId) return;
      const otherRadicals = this.data[key];
      const relation = determineRelationship(targetRadicals, otherRadicals);

      if (relation) {
        // Calculate complexity difference (e.g., 5 radicals vs 4 radicals)
        const complexityDiff = Math.abs(
          otherRadicals.length - targetRadicals.length,
        );

        // If 'child' (meaning 'key' is more complex than 'target'),
        // we strictly limit it to those that are only 1 step away (immediate supersets).
        // e.g. If target is '八' (2), we allow '分' (3), but skip '貧' (7).
        if (relation === 'child' && complexityDiff > 1) {
          return;
        }

        // Optional: If you also want to limit looking "down" (parents)
        // to only immediate parents, uncomment the line below.
        // Usually, seeing all roots is fine, so we can leave this commented.
        // if (relation === 'parent' && complexityDiff > 1) return;

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

  public hideNode(targetId: string) {
    // 1. Remove the target node immediately
    this.nodes = this.nodes.filter((n) => n.id !== targetId);

    // 2. Remove any links connected to that node
    // Note: D3 converts source/target to objects, so we check .id if it exists, else the string
    this.links = this.links.filter((l) => {
      const sourceId = (l.source as Node).id || l.source;
      const targetIdVal = (l.target as Node).id || l.target;
      return sourceId !== targetId && targetIdVal !== targetId;
    });

    // 3. Garbage Collection (Optional but Recommended)
    // Remove any non-root nodes that now have 0 connections (orphans)
    // because their only parent/neighbor was just hidden.
    let changed = true;
    while (changed) {
      changed = false;

      // Calculate active connections
      const connectedIds = new Set<string>();
      this.links.forEach((l) => {
        connectedIds.add((l.source as Node).id || (l.source as string));
        connectedIds.add((l.target as Node).id || (l.target as string));
      });

      // Filter out orphans (unless they are a ROOT, we keep roots)
      const initialCount = this.nodes.length;
      this.nodes = this.nodes.filter(
        (n) => n.type === 'root' || connectedIds.has(n.id),
      );

      if (this.nodes.length !== initialCount) changed = true;
    }

    // 4. Restart Physics
    this.updateSimulation();

    // Clear selection if we just hid the selected node
    if (this.selectedNodeId === targetId) {
      this.selectedNodeId = null;
      if (this.onNodeSelect) this.onNodeSelect(null);
    }
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

    // Clear using physical dimensions (this.canvas.width)
    // to ensure no artifacts remain on resize
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    // Transform system:
    // 1. Scale by DPR (so 1 D3 unit = 1 CSS pixel = X Physical pixels)
    this.ctx.scale(dpr, dpr);
    // 2. Apply Pan/Zoom transform
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
