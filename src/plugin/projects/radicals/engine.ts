import * as d3 from 'd3';
import type { FederatedPointerEvent } from 'pixi.js';
import { Application, Container, Graphics, Text } from 'pixi.js';
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

// --- HELPER FUNCTIONS ---

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

  // Rendering Layers
  private linkLayer: Graphics;
  private nodeLayer: Container;

  // State
  private width: number = 800;
  private height: number = 600;
  private data: Record<string, string[]>;
  private canvasElement: HTMLCanvasElement;
  private isInitialized: boolean = false;

  // Interaction State
  private hoveredNodeId: string | null = null;
  private selectedNodeId: string | null = null;
  private draggingNode: Node | null = null;

  // Physics
  private simulation!: d3.Simulation<Node, Link>;
  private nodes: Node[] = [];
  private links: Link[] = [];

  // Callbacks
  public onNodeSelect: ((node: Node | null) => void) | null = null;
  public onReady: (() => void) | null = null;

  constructor(
    canvasElement: HTMLCanvasElement,
    data: Record<string, string[]>,
  ) {
    this.data = data || {};
    this.canvasElement = canvasElement;

    this.app = new Application();

    // Create layers immediately
    this.linkLayer = new Graphics();
    this.nodeLayer = new Container();
  }

  public async init() {
    if (this.isInitialized) return;

    // 1. Measure Container
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
    this.viewport.drag().pinch().wheel().decelerate().clampZoom({
      minScale: 0.2, // Max Zoom Out (The "Kagen" or lower limit)
      maxScale: 3.0, // Max Zoom In (The "Jogen" or upper limit)
    });

    // 6. Init Physics
    this.simulation = d3
      .forceSimulation<Node, Link>()
      .velocityDecay(0.6)
      .force(
        'link',
        d3
          .forceLink<Node, Link>()
          .id((d: any) => d.id)
          .distance(80),
      )
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
    if (
      !this.isInitialized ||
      !this.app.renderer ||
      !this.canvasElement.parentElement
    )
      return;

    const parent = this.canvasElement.parentElement;
    const rect = parent.getBoundingClientRect();

    if (this.width === rect.width && this.height === rect.height) return;

    this.width = rect.width;
    this.height = rect.height;

    this.app.renderer.resize(this.width, this.height);

    if (this.viewport) {
      this.viewport.resize(this.width, this.height);
    }

    if (this.simulation) {
      this.simulation.force(
        'center',
        d3.forceCenter(this.width / 2, this.height / 2),
      );
      this.simulation.force('x', d3.forceX(this.width / 2).strength(0.05));
      this.simulation.force('y', d3.forceY(this.height / 2).strength(0.05));
      this.simulation.alpha(0.3).restart();
    }

    this.app.render();
  }

  public destroy() {
    this.simulation?.stop();
    this.app.destroy(true, { children: true });
  }

  // --- GRAPH LOGIC ---

  public search(kanji: string) {
    if (this.data[kanji]) {
      this.promoteToRoot(kanji);
    } else {
      alert('Kanji not found in dataset');
    }
  }

  public promoteToRoot(targetId: string) {
    this.nodes.forEach((n) => {
      if (n.type === 'root') n.type = 'standard';
    });

    let targetNode = this.nodes.find((n) => n.id === targetId);
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

    // We use a slight timeout or wait for the node's position to settle if it was just created.
    // However, since createNode gives a position, we can animate immediately.
    if (targetNode.x != null && targetNode.y != null) {
      this.viewport.animate({
        position: { x: targetNode.x, y: targetNode.y },
        scale: 1.5, // Optional: Zoom in slightly to focus
        time: 1000, // Duration in ms
        ease: 'easeInOutSine', // Smooth easing function
      });
    }
  }

  public expandNode(targetId: string) {
    const targetRadicals = this.data[targetId];
    if (!targetRadicals) {
      console.warn('No data for', targetId);
      return;
    }

    let targetNode = this.nodes.find((n) => n.id === targetId);
    if (!targetNode) {
      targetNode = this.createNode(
        targetId,
        this.nodes.length === 0 ? 'root' : 'standard',
      );
      this.nodes.push(targetNode);
    }
    targetNode.status = 'visible';

    Object.keys(this.data).forEach((key) => {
      if (key === targetId) return;

      const otherRadicals = this.data[key];
      const relation = determineRelationship(targetRadicals, otherRadicals);

      if (relation) {
        const complexityDiff = Math.abs(
          otherRadicals.length - targetRadicals.length,
        );
        if (relation === 'child' && complexityDiff > 1) return;

        let neighbor = this.nodes.find((n) => n.id === key);
        if (!neighbor) {
          neighbor = this.createNode(key, 'standard');
          neighbor.x = targetNode!.x;
          neighbor.y = targetNode!.y;
          neighbor.status = 'shadow';
          this.nodes.push(neighbor);
        }

        const linkExists = this.links.some((l) => {
          const s = (l.source as Node).id || l.source;
          const t = (l.target as Node).id || l.target;
          return (s === targetId && t === key) || (s === key && t === targetId);
        });

        if (!linkExists) {
          let src = targetId,
            tgt = key,
            rel = relation;
          if (relation === 'parent') {
            src = key;
            tgt = targetId;
            rel = 'child';
          }
          this.links.push({ source: src, target: tgt, relation: rel } as any);
        }
      }
    });

    this.syncVisuals();
    this.updateSimulation();
  }

  public hideNode(targetId: string) {
    this.nodes = this.nodes.filter((n) => n.id !== targetId);
    this.links = this.links.filter((l) => {
      const s = (l.source as Node).id || l.source;
      const t = (l.target as Node).id || l.target;
      return s !== targetId && t !== targetId;
    });

    const connectedIds = new Set<string>();
    this.links.forEach((l) => {
      connectedIds.add((l.source as Node).id || (l.source as string));
      connectedIds.add((l.target as Node).id || (l.target as string));
    });
    this.nodes = this.nodes.filter(
      (n) => n.type === 'root' || connectedIds.has(n.id),
    );

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
      y: this.height / 2 + (Math.random() - 0.5) * 10,
    };
  }

  private updateSimulation() {
    if (!this.simulation) return;
    this.simulation.nodes(this.nodes);
    (this.simulation.force('link') as d3.ForceLink<Node, Link>).links(
      this.links,
    );
    this.simulation.alpha(1).restart();
  }

  // --- VISUAL RENDERING ---

  private syncVisuals() {
    // 1. Cleanup Dead Nodes
    const liveIds = new Set(this.nodes.map((n) => n.id));
    for (let i = this.nodeLayer.children.length - 1; i >= 0; i--) {
      const child = this.nodeLayer.children[i] as Container;
      if (!liveIds.has(child.name)) {
        this.nodeLayer.removeChild(child);
        child.destroy({ children: true });
      }
    }

    // 2. Pre-calculate neighbors of the hovered node for "Ukibori" effect
    const hoveredNeighbors = new Set<string>();
    if (this.hoveredNodeId) {
      this.links.forEach((l) => {
        const s = (l.source as Node).id || (l.source as string);
        const t = (l.target as Node).id || (l.target as string);
        if (s === this.hoveredNodeId) hoveredNeighbors.add(t);
        if (t === this.hoveredNodeId) hoveredNeighbors.add(s);
      });
    }

    // 3. Create/Update Nodes
    this.nodes.forEach((node) => {
      let container = this.nodeLayer.getChildByName(node.id) as Container;

      if (!container) {
        container = new Container();
        container.name = node.id;
        container.eventMode = 'static';
        container.cursor = 'pointer';

        // Event Listeners
        container.on('pointerenter', () => this.onNodeHover(node.id));
        container.on('pointerleave', () => this.onNodeHover(null));
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
            fontFamily:
              '"Noto Sans JP", "Hiragino Kaku Gothic Pro", sans-serif',
            fontSize: BASE_SIZE * RESOLUTION_MULTIPLIER,
            fill: '#1e293b',
            align: 'center',
          },
        });

        label.scale.set(1 / RESOLUTION_MULTIPLIER);
        label.anchor.set(0.5);
        label.name = 'label';
        container.addChild(label);

        this.nodeLayer.addChild(container);
        node.gfx = container;
      }

      // Update Visuals
      const circle = container.getChildByName('circle') as Graphics;
      const label = container.getChildByName('label') as Text;

      const isSelected = this.selectedNodeId === node.id;
      const isHovered = this.hoveredNodeId === node.id;
      const isHoverNeighbor = hoveredNeighbors.has(node.id);

      // --- Fill Color ---
      let fillColor = 0xffffff;
      if (isSelected) fillColor = 0xeff6ff;
      else if (node.status === 'shadow') fillColor = 0xf1f5f9;
      else if (node.type === 'root') fillColor = 0xfee2e2;

      // --- Stroke Color ---
      let strokeColor = 0x3b82f6;
      if (isSelected) strokeColor = 0x2563eb;
      else if (node.type === 'root') strokeColor = 0xef4444;

      // --- Stroke Width & Alpha Logic ---
      // Default: 3 (Selected), 1 (Shadow), 2 (Standard)
      let strokeWidth = isSelected ? 3 : node.status === 'shadow' ? 1 : 2;

      // Ukibori: If Shadow AND (Hovered OR Connected to Hovered), bump visibility
      if (node.status === 'shadow' && (isHovered || isHoverNeighbor)) {
        strokeWidth = 2; // Make it look active
        strokeColor = 0x94a3b8; // Darker grey to stand out
      }

      let alpha = node.status === 'shadow' ? 0.6 : 1.0;
      if (node.status === 'shadow' && (isHovered || isHoverNeighbor)) {
        alpha = 1.0;
      }

      circle.clear();
      circle.beginPath();
      circle.roundRect(-20, -20, 40, 40, 20);
      circle.fill({ color: fillColor, alpha: 1 });
      circle.stroke({ width: strokeWidth, color: strokeColor, alpha });

      // Label Color Logic
      if (node.status === 'shadow' && (isHovered || isHoverNeighbor)) {
        label.style.fill = '#475569'; // Darker text for readability during hover
      } else {
        label.style.fill = node.status === 'shadow' ? '#94a3b8' : '#1e293b';
      }
    });
  }

  private renderTick() {
    if (!this.linkLayer) return;

    this.linkLayer.clear();

    // Sort links: Shadows bottom, Active middle, Hovered top
    this.links.sort((a, b) => {
      const aHovered =
        (a.source as Node).id === this.hoveredNodeId ||
        (a.target as Node).id === this.hoveredNodeId;
      const bHovered =
        (b.source as Node).id === this.hoveredNodeId ||
        (b.target as Node).id === this.hoveredNodeId;
      return Number(aHovered) - Number(bHovered);
    });

    this.links.forEach((link) => {
      const s = link.source as Node;
      const t = link.target as Node;

      if (s.x == null || s.y == null || t.x == null || t.y == null) return;

      const isShadow = s.status === 'shadow' || t.status === 'shadow';
      const isSibling = link.relation === 'sibling';
      const connectedToHover =
        s.id === this.hoveredNodeId || t.id === this.hoveredNodeId;

      let alpha = 1.0;
      let color = 0x64748b; // Slate 500
      let width = 2;

      if (isShadow) {
        if (connectedToHover) {
          // Ukibori: Hovering a node reveals its shadow connections
          alpha = 0.6;
          color = 0x94a3b8;
          width = 2;
        } else {
          // Hikaeme: Inactive shadows are very subtle
          alpha = 0.15;
          color = 0xcbd5e1;
          width = 1;
        }
      } else if (isSibling) {
        // Active siblings are secondary to parent/child
        alpha = 0.3;
        color = 0x94a3b8;
        width = 1;
      }

      this.linkLayer.moveTo(s.x, s.y);
      this.linkLayer.lineTo(t.x, t.y);
      this.linkLayer.stroke({ width, color, alpha });
    });

    this.nodes.forEach((node) => {
      if (node.gfx && node.x != null && node.y != null) {
        node.gfx.x = node.x;
        node.gfx.y = node.y;
      }
    });
  }

  // --- INTERACTION HANDLERS ---

  private onNodeHover(nodeId: string | null) {
    if (this.hoveredNodeId === nodeId) return;
    this.hoveredNodeId = nodeId;

    // Update visuals immediately without waiting for tick
    this.syncVisuals();
    this.renderTick();
    this.app.render();
  }

  private onDragStart(event: FederatedPointerEvent, node: Node) {
    if (!this.viewport) return;
    this.viewport.plugins.pause('drag');
    this.draggingNode = node;
    this.selectedNodeId = node.id;

    if (this.onNodeSelect) this.onNodeSelect(node);

    this.syncVisuals();
    if (this.simulation) this.simulation.alphaTarget(0.1).restart();

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
