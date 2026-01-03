import * as d3 from 'd3';
import type { FederatedPointerEvent, Ticker } from 'pixi.js';
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

// Interface for our new shooting stars
interface ShootingStar {
  gfx: Graphics;
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  dx: number;
  dy: number;
  currentX: number;
  currentY: number;
  progress: number; // 0.0 to 1.0
  speed: number; // progress increment per frame
  size: number;
  color: number;
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

// --- THEME & CONFIG CONSTANTS ---
const THEME = {
  background: '#020617', // Deep cosmos
  starRoot: '#fbbf24', // Amber/Gold (The Sun)
  starStandard: '#e0f2fe', // Sky Blue/White (Sirius)
  starShadow: '#64748b',
  textMain: '#f8fafc',
  textDim: '#475569',
  linkActive: '#94a3b8',
  linkDim: '#1e293b',
};

const SHOOTING_STAR_CONFIG = {
  colors: [0xffffff, 0xc0e0ff, 0xffd700], // White, icy blue, goldish
  minSpeed: 0.005,
  maxSpeed: 0.012, // Slightly slower max speed for more majestic movement
  minSize: 1.5,
  maxSize: 3,
  trailLengthMult: 25, // Slightly shorter tail so the head is more prominent
  spawnPadding: 400,
  worldBounds: 4000,
};

// --- THE ENGINE ---

export class KanjiGraphEngine {
  private app: Application;
  private viewport!: Viewport;

  // Rendering Layers
  private backgroundLayer: Container; // For static distant stars
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

  // Shooting Star State
  private activeShootingStars: ShootingStar[] = [];
  private shootingStarRarity: number;

  // Callbacks
  public onNodeSelect: ((node: Node | null) => void) | null = null;
  public onReady: (() => void) | null = null;

  constructor(
    canvasElement: HTMLCanvasElement,
    data: Record<string, string[]>,
    //CHANGED: Significantly lowered default rarity.
    // 0.0002 is roughly 1 star every 80-90 seconds at 60fps.
    shootingStarRarity: number = 0.0002,
  ) {
    this.data = data || {};
    this.canvasElement = canvasElement;
    this.shootingStarRarity = shootingStarRarity;

    this.app = new Application();

    // Create layers immediately
    this.backgroundLayer = new Container();
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
      antialias: true, // Important for smooth star trails
      autoDensity: true,
      resolution: window.devicePixelRatio || 1,
      background: THEME.background, // Dark Night Sky
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
    this.viewport.addChild(this.backgroundLayer); // Stars behind everything
    this.viewport.addChild(this.linkLayer);
    this.viewport.addChild(this.nodeLayer);

    // 5. Generate Distant Background Stars
    this.generateBackgroundStars();

    // 6. Init Shooting Star System
    this.initShootingStarSystem();

    // 7. Activate Camera
    this.viewport.drag().pinch().wheel().decelerate().clampZoom({
      minScale: 0.2,
      maxScale: 3.0,
    });

    // 8. Init Physics
    this.simulation = d3
      .forceSimulation<Node, Link>()
      .velocityDecay(0.6)
      .force(
        'link',
        d3
          .forceLink<Node, Link>()
          .id((d: any) => d.id)
          .distance(100), // Slightly longer distance for elegance
      )
      .force('charge', d3.forceManyBody().strength(-800))
      .force('collide', d3.forceCollide().radius(45).iterations(2))
      .force('center', d3.forceCenter(this.width / 2, this.height / 2))
      .force('x', d3.forceX(this.width / 2).strength(0.04))
      .force('y', d3.forceY(this.height / 2).strength(0.04));

    this.simulation.on('tick', () => this.renderTick());

    this.isInitialized = true;

    if (this.onReady) this.onReady();
  }

  /**
   * Generates a static field of tiny stars in the background layer
   * to create the "Seiza" (Constellation) atmosphere.
   */
  private generateBackgroundStars() {
    const starGfx = new Graphics();
    const count = 300;
    const range = SHOOTING_STAR_CONFIG.worldBounds; // Large area so we can pan around

    for (let i = 0; i < count; i++) {
      const x = (Math.random() - 0.5) * range + this.width / 2;
      const y = (Math.random() - 0.5) * range + this.height / 2;
      const size = Math.random() * 2 + 0.5;
      const alpha = Math.random() * 0.5 + 0.1;

      starGfx.circle(x, y, size);
      starGfx.fill({ color: 0xffffff, alpha });
    }
    this.backgroundLayer.addChild(starGfx);
  }

  // --- SHOOTING STAR SYSTEM ---

  private initShootingStarSystem() {
    // Hook into Pixi's main ticker loop for smooth background animation separate from D3 physics
    this.app.ticker.add((ticker) => this.updateShootingStars(ticker));
  }

  private spawnShootingStar() {
    const cfg = SHOOTING_STAR_CONFIG;
    const range = cfg.worldBounds;
    const padding = cfg.spawnPadding;
    const centerX = this.width / 2;
    const centerY = this.height / 2;

    // 1. Determine Start Point (somewhere far outside bounds)
    const startAngle = Math.random() * Math.PI * 2;
    const startDistance = range / 2 + padding;
    const startX = centerX + Math.cos(startAngle) * startDistance;
    const startY = centerY + Math.sin(startAngle) * startDistance;

    // 2. Determine End Point (somewhere on the opposite side, crossing near center)
    const endAngle = startAngle + Math.PI + (Math.random() - 0.5); // +/- ~30 degrees variation
    const endDistance = range / 2 + padding;
    const endX = centerX + Math.cos(endAngle) * endDistance;
    const endY = centerY + Math.sin(endAngle) * endDistance;

    // 3. Setup Properties
    const dx = endX - startX;
    const dy = endY - startY;
    const size = cfg.minSize + Math.random() * (cfg.maxSize - cfg.minSize);
    const speed = cfg.minSpeed + Math.random() * (cfg.maxSpeed - cfg.minSpeed);
    const color = cfg.colors[Math.floor(Math.random() * cfg.colors.length)];

    const gfx = new Graphics();
    // Add to background layer so it's behind graph nodes but moves with camera panning
    this.backgroundLayer.addChild(gfx);

    this.activeShootingStars.push({
      gfx,
      startX,
      startY,
      endX,
      endY,
      dx,
      dy,
      currentX: startX,
      currentY: startY,
      progress: 0,
      speed,
      size,
      color,
    });
  }

  private updateShootingStars(ticker: Ticker) {
    // 1. Chance to spawn a new one
    // The constructor default is now much lower for rarity.
    if (Math.random() < this.shootingStarRarity * ticker.deltaTime) {
      this.spawnShootingStar();
    }

    // 2. Update and Render active stars
    const cfg = SHOOTING_STAR_CONFIG;

    // Iterate backwards so we can splice safely
    for (let i = this.activeShootingStars.length - 1; i >= 0; i--) {
      const star = this.activeShootingStars[i];

      // Increment progress based on speed and ticker time scaling
      star.progress += star.speed * ticker.deltaTime;

      if (star.progress >= 1) {
        // Star finished its journey
        star.gfx.destroy();
        this.activeShootingStars.splice(i, 1);
        continue;
      }

      // Calculate realistic fading.
      // Use a sine wave based on progress.
      // Math.sin(0) is 0. Math.sin(PI/2) is 1. Math.sin(PI) is 0.
      // This creates a smooth curve starting at invisible, brightest in the middle, ending invisible.
      const cycleAlpha = Math.sin(star.progress * Math.PI);

      // Don't bother rendering if it's basically invisible
      if (cycleAlpha < 0.01) {
        star.gfx.clear();
        continue;
      }

      // Calculate new position based on linear interpolation
      star.currentX = star.startX + star.dx * star.progress;
      star.currentY = star.startY + star.dy * star.progress;

      // Calculate trail tail position backwards along the vector
      const range = cfg.worldBounds;
      const tailEndX =
        star.currentX - (star.dx / range) * cfg.trailLengthMult * star.size;
      const tailEndY =
        star.currentY - (star.dy / range) * cfg.trailLengthMult * star.size;

      // RENDER
      star.gfx.clear();

      // Draw Tail (Fading Line)
      star.gfx.moveTo(tailEndX, tailEndY);
      star.gfx.lineTo(star.currentX, star.currentY);
      star.gfx.stroke({
        width: star.size * 1.5,
        color: star.color,
        // CHANGED: Multiply base trail alpha by the lifecycle alpha
        alpha: 0.3 * cycleAlpha,
        cap: 'round',
      });

      // Draw Head (Bright glowing center)
      // Inner core (white hot)
      star.gfx.circle(star.currentX, star.currentY, star.size * 0.8);
      // CHANGED: Apply lifecycle alpha
      star.gfx.fill({ color: 0xffffff, alpha: 1.0 * cycleAlpha });

      // Outer glow (colored)
      star.gfx.circle(star.currentX, star.currentY, star.size * 2.0);
      // CHANGED: Apply lifecycle alpha
      star.gfx.fill({ color: star.color, alpha: 0.4 * cycleAlpha });
    }
  }

  // =========================================
  // === EXISTING CLASS METHODS BELOW HERE ===
  // =========================================

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
    // Clean up shooting star ticker
    this.app.ticker.remove(this.updateShootingStars, this);
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

    if (targetNode.x != null && targetNode.y != null) {
      this.viewport.animate({
        position: { x: targetNode.x, y: targetNode.y },
        time: 1000,
        ease: 'easeInOutSine',
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

    // 2. Pre-calculate neighbors for Ukibori
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
        circle.name = 'starBody';
        container.addChild(circle);

        const RESOLUTION_MULTIPLIER = 2;
        const BASE_SIZE = 16;

        const label = new Text({
          text: node.id,
          style: {
            fontFamily:
              '"Noto Sans JP", "Hiragino Kaku Gothic Pro", sans-serif',
            fontSize: BASE_SIZE * RESOLUTION_MULTIPLIER,
            fill: THEME.textMain,
            align: 'center',
            fontWeight: 'bold',
          },
        });

        label.scale.set(1 / RESOLUTION_MULTIPLIER);
        label.anchor.set(0.5);
        label.y = 28; // Position label below the star
        label.name = 'label';
        container.addChild(label);

        this.nodeLayer.addChild(container);
        node.gfx = container;
      }

      // Update Visuals
      const starBody = container.getChildByName('starBody') as Graphics;
      const label = container.getChildByName('label') as Text;

      const isSelected = this.selectedNodeId === node.id;
      const isHovered = this.hoveredNodeId === node.id;
      const isHoverNeighbor = hoveredNeighbors.has(node.id);

      // --- Palette Selection ---
      // Defaults
      let coreColor = THEME.starStandard;
      let glowColor = THEME.starStandard;
      let coreRadius = 6;
      let alpha = 1.0;

      if (node.type === 'root') {
        coreColor = THEME.starRoot;
        glowColor = '#f59e0b'; // Slightly darker orange/gold
        coreRadius = 10;
      } else if (node.status === 'shadow') {
        coreColor = THEME.starShadow;
        glowColor = THEME.starShadow;
        coreRadius = 3;
        alpha = 0.5;
      }

      // Interaction Overrides
      if (isSelected) {
        coreRadius += 2;
        glowColor = '#ffffff'; // White hot
      }

      if (node.status === 'shadow' && (isHovered || isHoverNeighbor)) {
        // Shadow star lighting up when looked at
        coreColor = THEME.starStandard;
        glowColor = '#ffffff';
        alpha = 1.0;
        coreRadius = 5;
      } else if (
        node.status === 'visible' &&
        this.hoveredNodeId &&
        !isHovered &&
        !isHoverNeighbor
      ) {
        // Dim unrelated visible nodes slightly to focus on constellation
        alpha = 0.3;
      }

      starBody.clear();

      // 1. Hitbox
      // Draw a transparent circle first. This expands the clickable area
      // without affecting the visual rendering.
      // We set a minimum radius of 25px for easy clicking.
      starBody.circle(0, 0, 25);
      // Alpha 0 is invisible but still registers as a hit in the container
      starBody.fill({ color: 0x000000, alpha: 0 });

      // 2. Glow (Outer Haze)
      if (node.status !== 'shadow' || isHovered || isHoverNeighbor) {
        starBody.circle(0, 0, coreRadius * 3);
        starBody.fill({ color: glowColor, alpha: 0.2 * alpha });

        starBody.circle(0, 0, coreRadius * 1.5);
        starBody.fill({ color: glowColor, alpha: 0.4 * alpha });
      }

      // 3. Core (Solid Star)
      starBody.circle(0, 0, coreRadius);
      starBody.fill({ color: coreColor, alpha: alpha });

      // 4. Optional: Tiny white center for extra shine
      if (node.type === 'root' || isSelected) {
        starBody.circle(0, 0, coreRadius * 0.4);
        starBody.fill({ color: '#ffffff', alpha: 1 });
      }

      // Label Styling
      // Checks if the node is hovered, a neighbor, or selected.
      if (
        node.status === 'shadow' &&
        !isHovered &&
        !isHoverNeighbor &&
        !isSelected
      ) {
        label.visible = false; // Hide names of distant shadow stars
      } else {
        label.visible = true;
        label.style.fill = isHovered || isSelected ? '#ffffff' : THEME.textMain;
        label.alpha = alpha;
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
      const connectedToHover =
        s.id === this.hoveredNodeId || t.id === this.hoveredNodeId;

      let alpha = 1.0;
      let color = THEME.linkActive;
      let width = 1;

      if (this.hoveredNodeId && !connectedToHover) {
        // If focusing on a constellation, fade out unrelated lines
        alpha = 0.05;
      } else if (isShadow) {
        if (connectedToHover) {
          // Reveal the potential connection
          alpha = 0.5;
          color = '#ffffff';
          width = 1;
        } else {
          // Hidden in the dark
          alpha = 0.0;
        }
      } else {
        // Standard visible connection (constellation line)
        alpha = 0.3;
        width = 1.5;
      }

      if (alpha > 0) {
        this.linkLayer.moveTo(s.x, s.y);
        this.linkLayer.lineTo(t.x, t.y);
        this.linkLayer.stroke({ width, color, alpha });
      }
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
