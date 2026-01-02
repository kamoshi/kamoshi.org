import * as d3 from 'd3';
import { Application, Container, Graphics, Text, FederatedPointerEvent } from 'pixi.js';
import { Viewport } from 'pixi-viewport';

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
  gfx?: Container;
}

export interface Link extends d3.SimulationLinkDatum<Node> {
  source: string | Node;
  target: string | Node;
  relation: RelationType;
}

// --- HELPER FUNCTIONS (MUST BE DEFINED HERE) ---

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
  private app: Application;
  private viewport!: Viewport;

  // Rendering Layers - Initialized in constructor
  private linkLayer: Graphics;
  private nodeLayer: Container;

  // State
  private width: number = 800;
  private height: number = 600;
  private data: Record<string, string[]>;
  private canvasElement: HTMLCanvasElement;
  private isInitialized: boolean = false;

  // Physics
  private simulation!: d3.Simulation<Node, Link>;
  private nodes: Node[] = [];
  private links: Link[] = [];

  // Interaction
  public onNodeSelect: ((node: Node | null) => void) | null = null;
  public onReady: (() => void) | null = null;
  private selectedNodeId: string | null = null;
  private draggingNode: Node | null = null;

  constructor(canvasElement: HTMLCanvasElement, data: Record<string, string[]>) {
    this.data = data || {};
    this.canvasElement = canvasElement;

    this.app = new Application();

    // SAFEGUARD: Create layers immediately so accessing them never throws error
    this.linkLayer = new Graphics();
    this.nodeLayer = new Container();
  }

  public async init() {
    if (this.isInitialized) return;

    // 1. Measure Canvas Parent instead of canvas itself
    // We want the size of the container, not the current size of the canvas element
    const parent = this.canvasElement.parentElement;
    const rect = parent?.getBoundingClientRect();
    this.width = rect?.width || 800;
    this.height = rect?.height || 600;

    // 2. Init Pixi Application
    await this.app.init({
      canvas: this.canvasElement,
      width: this.width,
      height: this.height,
      antialias: true,
      autoDensity: true,
      resolution: window.devicePixelRatio || 1,
      background: '#f8fafc',
    });

    // 3. Setup Viewport
    this.viewport = new Viewport({
      screenWidth: this.width,
      screenHeight: this.height,
      worldWidth: 2000,
      worldHeight: 2000,
      events: this.app.renderer.events,
    });

    this.app.stage.addChild(this.viewport);

    // 4. Attach Layers
    this.viewport.addChild(this.linkLayer);
    this.viewport.addChild(this.nodeLayer);

    // 5. Activate Camera
    this.viewport.drag().pinch().wheel().decelerate();

    // 6. Init Physics
    this.simulation = d3
      .forceSimulation<Node, Link>()
      .velocityDecay(0.6)
      .force('link', d3.forceLink<Node, Link>().id((d: any) => d.id).distance(80))
      .force('charge', d3.forceManyBody().strength(-800))
      .force('collide', d3.forceCollide().radius(40).iterations(2))
      .force('center', d3.forceCenter(this.width / 2, this.height / 2))
      .force('x', d3.forceX(this.width / 2).strength(0.05))
      .force('y', d3.forceY(this.height / 2).strength(0.05));

    this.simulation.on('tick', () => this.renderTick());

    this.isInitialized = true;

    if (this.onReady) this.onReady();
  }

  public resize() {
    // 1. Safety check
    if (!this.isInitialized || !this.app.renderer || !this.canvasElement.parentElement) return;

    // 2. Measure the Container (The Source of Truth)
    const parent = this.canvasElement.parentElement;
    const rect = parent.getBoundingClientRect();

    // Check if size actually changed to avoid unnecessary computations
    if (this.width === rect.width && this.height === rect.height) return;

    this.width = rect.width;
    this.height = rect.height;

    // 3. Tell Pixi to Resize the Renderer
    this.app.renderer.resize(this.width, this.height);

    // 4. Update Viewport World Dimensions
    if (this.viewport) {
      this.viewport.resize(this.width, this.height);
    }

    // 5. Update D3 Simulation Forces
    if (this.simulation) {
      // Re-center the gravity to the new middle of the screen
      this.simulation.force('center', d3.forceCenter(this.width / 2, this.height / 2));

      // Update the gentle X/Y positioning forces
      this.simulation.force('x', d3.forceX(this.width / 2).strength(0.05));
      this.simulation.force('y', d3.forceY(this.height / 2).strength(0.05));

      // "Wake up" the simulation slightly so nodes drift to new center
      this.simulation.alpha(0.3).restart();
    }

    this.app.render();
  }

  public destroy() {
    this.simulation?.stop();
    this.app.destroy(true, { children: true });
  }

  // --- LOGIC ---

  public search(kanji: string) {
    if (this.data[kanji]) {
      this.promoteToRoot(kanji);
    } else {
      alert('Kanji not found in dataset');
    }
  }

  public promoteToRoot(targetId: string) {
    this.nodes.forEach(n => { if(n.type === 'root') n.type = 'standard'; });

    let targetNode = this.nodes.find(n => n.id === targetId);
    if (targetNode) {
      targetNode.type = 'root';
      targetNode.status = 'visible';
    } else {
      targetNode = this.createNode(targetId, 'root');
      this.nodes.push(targetNode);
    }

    this.selectedNodeId = targetId;
    if (this.onNodeSelect) this.onNodeSelect(targetNode);

    this.expandNode(targetId);
  }

  public expandNode(targetId: string) {
    const targetRadicals = this.data[targetId];
    if (!targetRadicals) {
        console.warn("No data for", targetId);
        return;
    }

    let targetNode = this.nodes.find(n => n.id === targetId);
    if (!targetNode) {
        targetNode = this.createNode(targetId, this.nodes.length === 0 ? 'root' : 'standard');
        this.nodes.push(targetNode);
    }
    targetNode.status = 'visible';

    // Neighbor Discovery
    Object.keys(this.data).forEach(key => {
        if (key === targetId) return;

        const otherRadicals = this.data[key];
        const relation = determineRelationship(targetRadicals, otherRadicals);

        if (relation) {
            const complexityDiff = Math.abs(otherRadicals.length - targetRadicals.length);
            if (relation === 'child' && complexityDiff > 1) return;

            let neighbor = this.nodes.find(n => n.id === key);
            if (!neighbor) {
                neighbor = this.createNode(key, 'standard');
                neighbor.x = targetNode!.x;
                neighbor.y = targetNode!.y;
                neighbor.status = 'shadow';
                this.nodes.push(neighbor);
            }

            const linkExists = this.links.some(l => {
                const s = (l.source as Node).id || l.source;
                const t = (l.target as Node).id || l.target;
                return (s === targetId && t === key) || (s === key && t === targetId);
            });

            if (!linkExists) {
                let src = targetId, tgt = key, rel = relation;
                if (relation === 'parent') { src = key; tgt = targetId; rel = 'child'; }
                this.links.push({ source: src, target: tgt, relation: rel } as any);
            }
        }
    });

    this.syncVisuals();
    this.updateSimulation();
  }

  public hideNode(targetId: string) {
    this.nodes = this.nodes.filter(n => n.id !== targetId);
    this.links = this.links.filter(l => {
        const s = (l.source as Node).id || l.source;
        const t = (l.target as Node).id || l.target;
        return s !== targetId && t !== targetId;
    });

    const connectedIds = new Set<string>();
    this.links.forEach(l => {
        connectedIds.add((l.source as Node).id || l.source as string);
        connectedIds.add((l.target as Node).id || l.target as string);
    });
    this.nodes = this.nodes.filter(n => n.type === 'root' || connectedIds.has(n.id));

    if (this.selectedNodeId === targetId) {
        this.selectedNodeId = null;
        if (this.onNodeSelect) this.onNodeSelect(null);
    }
    this.syncVisuals();
    this.updateSimulation();
  }

  // --- INTERNAL UTILS ---

  private createNode(id: string, type: 'root' | 'standard'): Node {
      return {
          id,
          radicals: this.data[id] || [],
          status: 'visible',
          type,
          x: this.width / 2 + (Math.random() - 0.5) * 10,
          y: this.height / 2 + (Math.random() - 0.5) * 10
      };
  }

  private updateSimulation() {
    if (!this.simulation) return;
    this.simulation.nodes(this.nodes);
    (this.simulation.force('link') as d3.ForceLink<Node, Link>).links(this.links);
    this.simulation.alpha(1).restart();
  }

  // --- VISUAL RENDERING ---

  private syncVisuals() {
      // 1. Cleanup
      const liveIds = new Set(this.nodes.map(n => n.id));
      for (let i = this.nodeLayer.children.length - 1; i >= 0; i--) {
          const child = this.nodeLayer.children[i] as Container;
          if (!liveIds.has(child.name)) {
              this.nodeLayer.removeChild(child);
              child.destroy({ children: true });
          }
      }

      // 2. Create/Update
      this.nodes.forEach(node => {
          let container = this.nodeLayer.getChildByName(node.id) as Container;

          if (!container) {
              container = new Container();
              container.name = node.id;
              container.eventMode = 'static';
              container.cursor = 'pointer';
              container.on('pointerdown', (e) => this.onDragStart(e, node));
              container.on('pointerup', (e) => this.onDragEnd());
              container.on('pointerupoutside', (e) => this.onDragEnd());

              const circle = new Graphics();
              circle.name = 'circle';
              container.addChild(circle);

              const RESOLUTION_MULTIPLIER = 4;
              const BASE_SIZE = 16;

              const label = new Text({
                text: node.id,
                style: {
                  fontFamily: '"Noto Sans JP", "Hiragino Kaku Gothic Pro", sans-serif', // Better Kanji fonts
                  // 2. Render the font HUGE
                  fontSize: BASE_SIZE * RESOLUTION_MULTIPLIER,
                  fill: '#1e293b',
                  align: 'center'
                }
              });

              // 3. Shrink the object back down to normal size
              // (1 / 4 = 0.25 scale)
              label.scale.set(1 / RESOLUTION_MULTIPLIER);

              label.anchor.set(0.5);
              label.name = 'label';
              container.addChild(label);

              this.nodeLayer.addChild(container);
              node.gfx = container;
          }

          // Update Props
          const circle = container.getChildByName('circle') as Graphics;
          const label = container.getChildByName('label') as Text;
          const isSelected = this.selectedNodeId === node.id;

          let fillColor = 0xffffff;
          if (isSelected) fillColor = 0xeff6ff;
          else if (node.status === 'shadow') fillColor = 0xf1f5f9;
          else if (node.type === 'root') fillColor = 0xfee2e2;

          let strokeColor = 0x3b82f6;
          if (isSelected) strokeColor = 0x2563eb;
          else if (node.type === 'root') strokeColor = 0xef4444;

          const strokeWidth = isSelected ? 3 : (node.status === 'shadow' ? 1 : 2);
          const alpha = node.status === 'shadow' ? 0.6 : 1.0;

          circle.clear();
          circle.beginPath();
          circle.roundRect(-20, -20, 40, 40, 20);
          circle.fill({ color: fillColor, alpha: 1 });
          circle.stroke({ width: strokeWidth, color: strokeColor, alpha });

          label.style.fill = node.status === 'shadow' ? '#94a3b8' : '#1e293b';
      });
  }

  private renderTick() {
      if (!this.linkLayer) return;

      this.linkLayer.clear();

      this.links.forEach(link => {
          const s = link.source as Node;
          const t = link.target as Node;

          if (s.x == null || s.y == null || t.x == null || t.y == null) return;

          const isSibling = link.relation === 'sibling';
          const alpha = isSibling ? 0.3 : 1.0;
          const color = isSibling ? 0x94a3b8 : 0x64748b;
          const width = isSibling ? 1 : 2;

          this.linkLayer.moveTo(s.x, s.y);
          this.linkLayer.lineTo(t.x, t.y);
          this.linkLayer.stroke({ width, color, alpha });
      });

      this.nodes.forEach(node => {
          if (node.gfx && node.x != null && node.y != null) {
              node.gfx.x = node.x;
              node.gfx.y = node.y;
          }
      });
  }

  // --- INTERACTION ---
  private onDragStart(event: FederatedPointerEvent, node: Node) {
      if (!this.viewport) return;
      this.viewport.plugins.pause('drag');
      this.draggingNode = node;
      this.selectedNodeId = node.id;
      if (this.onNodeSelect) this.onNodeSelect(node);
      this.syncVisuals();
      if (this.simulation) this.simulation.alphaTarget(0.3).restart();
      node.fx = node.x;
      node.fy = node.y;
      this.viewport.on('pointermove', this.onDragMove, this);
  }

  private onDragMove(event: FederatedPointerEvent) {
      if (!this.draggingNode || !this.viewport) return;
      const worldPos = this.viewport.toWorld(event.global);
      this.draggingNode.fx = worldPos.x;
      this.draggingNode.fy = worldPos.y;
  }

  private onDragEnd() {
      if (!this.draggingNode || !this.viewport) return;
      this.viewport.plugins.resume('drag');
      if (this.simulation) this.simulation.alphaTarget(0);
      this.draggingNode.fx = null;
      this.draggingNode.fy = null;
      this.draggingNode = null;
      this.viewport.off('pointermove', this.onDragMove, this);
  }
}
