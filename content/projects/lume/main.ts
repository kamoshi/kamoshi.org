import { css, html, LitElement } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { wasmCallbacks, wasmReady, wasmToJs, wasmToLua } from './wasm';
import type { Output, Tab } from './x-output';
import './x-editor';
import './x-output';

// ── Sample ─────────────────────────────────────────────────────────────────

const SAMPLE = `\
-- ADT: a shape with multiple constructors
type Shape =
  | Circle   { radius: Num }
  | Rect     { width: Num, height: Num }
  | Triangle { base: Num, height: Num }

let pi = 3.14159

-- Pattern matching on ADT
let area : Shape -> Num =
  | Circle   { radius }         -> pi * radius * radius
  | Rect     { width, height }  -> width * height
  | Triangle { base, height }   -> base * height / 2

let describe : Shape -> Text =
  | Circle   { radius }         -> "circle r="    ++ show radius
  | Rect     { width, height }  -> "rect "         ++ show width ++ "x" ++ show height
  | Triangle { base, height }   -> "triangle b="   ++ show base  ++ " h=" ++ show height

-- Curried scaling function
let scale : Num -> Shape -> Shape = f ->
  | Circle   { radius }        -> Circle   { radius: radius * f }
  | Rect     { width, height } -> Rect     { width: width * f, height: height * f }
  | Triangle { base, height }  -> Triangle { base: base * f,   height: height * f }

-- Row polymorphism: works on any record with a \`score\` field
let passed : { score: Num, .. } -> Bool = { score, .. } -> score >= 60

-- Guards in pattern matching
let grade : Num -> Text =
  | s if s >= 90 -> "A"
  | s if s >= 75 -> "B"
  | s if s >= 60 -> "C"
  | _            -> "Fail"

-- Safe division returning a Result
let safeDivide : Num -> Num -> Result Num Text = a -> b ->
  if b == 0
    then Err { reason: "division by zero" }
    else Ok { value: a / b }

-- Chaining Results with ?>
let halfThenThird = n ->
  safeDivide n 2 ?> (half -> safeDivide half 3)

-- List processing with pipe operator
let scores = [85, 92, 67, 45, 78, 95, 55, 88]

let passing  = filter (s -> s >= 60) scores
let top      = filter (s -> s >= 85) scores
let graded   = map grade scores
let total    = sum scores
let avg      = average scores

-- Shapes pipeline
let shapes = [
  Circle   { radius: 3 },
  Rect     { width: 4, height: 5 },
  Triangle { base: 6, height: 8 },
]

let areas   = map area shapes
let doubled = map (scale 2) shapes
let labels  = map describe shapes

{
  area,
  describe,
  scale,
  grade,
  halfThenThird,
  passing,
  top,
  graded,
  total,
  avg,
  areas,
  labels,
}
`;

// ── Transpile ──────────────────────────────────────────────────────────────

function transpile(src: string, tab: Tab): Output {
  try {
    const code = tab === 'js' ? wasmToJs(src) : wasmToLua(src);
    return { kind: 'ok', code: String(code) };
  } catch (e) {
    return { kind: 'err', message: String(e) };
  }
}

// ── Orchestrator ───────────────────────────────────────────────────────────

@customElement('lume-editor')
export class LumeEditor extends LitElement {
  static styles = css`
    :host {
      display: block;
      height: 100%;
    }

    .split {
      display: grid;
      grid-template-columns: 1fr 1fr;
      height: 100%;
      gap: 0.5rem;
    }

    x-editor, x-output {
      border: 1px solid var(--c-border, #d0d0d0);
      border-radius: 4px;
      min-width: 0;
      min-height: 0;
      overflow: hidden;
    }
  `;

  @state() private tab: Tab = 'js';
  @state() private output: Output = { kind: 'ok', code: '' };
  private src = SAMPLE;

  firstUpdated() {
    const doCompile = () => {
      this.output = transpile(this.src, this.tab);
    };

    if (wasmReady) {
      doCompile();
    } else {
      wasmCallbacks.push(doCompile);
    }
  }

  private onSourceChange(e: CustomEvent<string>) {
    this.src = e.detail;
    if (wasmReady) this.output = transpile(this.src, this.tab);
  }

  private onTabChange(e: CustomEvent<Tab>) {
    this.tab = e.detail;
    if (wasmReady) this.output = transpile(this.src, this.tab);
  }

  render() {
    return html`
      <div class="split">
        <x-editor
          @lume-change=${this.onSourceChange}
          .initialDoc=${SAMPLE}
        ></x-editor>
        <x-output
          .output=${this.output}
          @tab-change=${this.onTabChange}
        ></x-output>
      </div>
    `;
  }
}
