import { LitElement, html, css, } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { repeat } from 'lit/directives/repeat.js';

// ─── Types ────────────────────────────────────────────────────────────────────

interface FontEntry {
  id: string;
  name: string;
  css: string;
  elo: number;
  matches: number;
}

// ─── Shared Data ──────────────────────────────────────────────────────────────

const FONTS_DATA: Omit<FontEntry, 'elo' | 'matches'>[] = [
  { id: 'system-ui',           name: 'System UI',           css: 'system-ui, sans-serif' },
  { id: 'transitional',        name: 'Transitional',        css: "Charter, 'Bitstream Charter', 'Sitka Text', Cambria, serif" },
  { id: 'old-style',           name: 'Old Style',           css: "'Iowan Old Style', 'Palatino Linotype', 'URW Palladio L', P052, serif" },
  { id: 'humanist',            name: 'Humanist',            css: "Seravek, 'Gill Sans Nova', Ubuntu, Calibri, 'DejaVu Sans', source-sans-pro, sans-serif" },
  { id: 'geometric-humanist',  name: 'Geometric Humanist',  css: "Avenir, Montserrat, Corbel, 'URW Gothic', source-sans-pro, sans-serif" },
  { id: 'classical-humanist',  name: 'Classical Humanist',  css: "Optima, Candara, 'Noto Sans', source-sans-pro, sans-serif" },
  { id: 'neo-grotesque',       name: 'Neo-Grotesque',       css: "Inter, Roboto, 'Helvetica Neue', 'Arial Nova', 'Nimbus Sans', Arial, sans-serif" },
  { id: 'monospace-slab',      name: 'Monospace Slab Serif',css: "'Nimbus Mono PS', 'Courier New', monospace" },
  { id: 'monospace-code',      name: 'Monospace Code',      css: "ui-monospace, 'Cascadia Code', 'Source Code Pro', Menlo, Consolas, 'DejaVu Sans Mono', monospace" },
  { id: 'industrial',          name: 'Industrial',          css: "Bahnschrift, 'DIN Alternate', 'Franklin Gothic Medium', 'Nimbus Sans Narrow', sans-serif-condensed, sans-serif" },
  { id: 'rounded-sans',        name: 'Rounded Sans',        css: "ui-rounded, 'Hiragino Maru Gothic ProN', Quicksand, Comfortaa, Manjari, 'Arial Rounded MT Bold', Calibri, source-sans-pro, sans-serif" },
  { id: 'slab-serif',          name: 'Slab Serif',          css: "Rockwell, 'Rockwell Nova', 'Roboto Slab', 'DejaVu Serif', 'Sitka Small', serif" },
  { id: 'antique',             name: 'Antique',             css: "Superclarendon, 'Bookman Old Style', 'URW Bookman', 'URW Bookman L', 'Georgia Pro', Georgia, serif" },
  { id: 'didone',              name: 'Didone',              css: "Didot, 'Bodoni MT', 'Noto Serif Display', 'URW Palladio L', P052, Sylfaen, serif" },
  { id: 'handwritten',         name: 'Handwritten',         css: "'Segoe Print', 'Bradley Hand', Chilanka, TSCu_Comic, casual, cursive" },
];

const DEFAULT_TEXT =
  'The quick brown fox jumps over the lazy dog. Sphinx of black quartz, judge my vow.\n\n' +
  'Typography is the art and technique of arranging type to make written language legible, readable, and appealing. ' +
  'The arrangement of type involves selecting typefaces, point sizes, line lengths, line-spacing, and letter-spacing.';

function freshFonts(): FontEntry[] {
  return FONTS_DATA.map(f => ({ ...f, elo: 1200, matches: 0 }));
}

function calcElo(winner: FontEntry, loser: FontEntry, K = 32) {
  const expW = 1 / (1 + Math.pow(10, (loser.elo - winner.elo) / 400));
  const expL = 1 / (1 + Math.pow(10, (winner.elo - loser.elo) / 400));
  return {
    winnerElo: Math.round(winner.elo + K * (1 - expW)),
    loserElo:  Math.round(loser.elo  + K * (0 - expL)),
  };
}

function pickPair(fonts: FontEntry[]): [string, string] {
  const sorted = [...fonts].sort((a, b) => a.matches - b.matches);
  const fontA  = sorted[0];
  const others = fonts.filter(f => f.id !== fontA.id)
                      .sort((a, b) => Math.abs(a.elo - fontA.elo) - Math.abs(b.elo - fontA.elo));
  const pool = others.slice(0, 3);
  const fontB = pool[Math.floor(Math.random() * pool.length)];
  return Math.random() > 0.5 ? [fontA.id, fontB.id] : [fontB.id, fontA.id];
}

// ─── Shared event names ───────────────────────────────────────────────────────

const FONTS_CHANGED = 'fonts-changed';
const TEXT_CHANGED  = 'font-text-changed';

// ─── Component: <font-header> ─────────────────────────────────────────────────

@customElement('font-header')
export class FontHeader extends LitElement {
  static styles = css`
    :host {
      display: block;
      font-family: Georgia, 'Times New Roman', serif;
      --ink:    #1a1410;
      --rule:   #d4c9b8;
      --accent: #b5451b;
      --muted:  #7a6e62;
    }

    header { text-align: center; margin-top: 2rem; margin-bottom: 2.5rem; }

    .desc {
      font-size: 0.95rem;
      color: var(--ink);
      line-height: 1.7;
      margin: 1rem 0 0;
    }

    .desc a { color: var(--accent); }

    .masthead {
      font-size: clamp(2rem, 6vw, 4rem);
      font-weight: 900;
      letter-spacing: -0.03em;
      line-height: 1;
      color: var(--ink);
      margin: 0 0 0.5rem;
      font-variant: small-caps;
    }

    .rule-line {
      width: 100%;
      height: 3px;
      background: linear-gradient(to right, transparent, var(--accent), transparent);
      border: none;
      margin: 0.75rem 0;
    }


  `;

  render() {
    return html`
      <header>
        <h1 class="masthead">Font Stack Showdown</h1>
        <hr class="rule-line" />
        <p class="desc">
          Which system font stack looks best to your eye? This tool pits them
          against each other in a head-to-head vote. After enough matches, the
          <a href="https://en.wikipedia.org/wiki/Elo_rating_system">Elo rating system</a>
          settles on a ranking that reflects your genuine preferences. All fonts
          are system stacks: no web fonts are downloaded, so what you see
          reflects what's actually installed on your device. Results will differ
          between operating systems.
        </p>
      </header>
    `;
  }
}

// ─── Component: <font-textbox> ────────────────────────────────────────────────

@customElement('font-textbox')
export class FontTextbox extends LitElement {

  @state() private sampleText: string = DEFAULT_TEXT;

  static styles = css`
    :host {
      display: block;
      font-family: Georgia, 'Times New Roman', serif;
      --ink:     #1a1410;
      --rule:    #d4c9b8;
      --muted:   #7a6e62;
      --card-bg: #ffffff;
    }

    * { box-sizing: border-box; }

    .textarea-wrap {
      border: 1px solid var(--rule);
      border-radius: 4px;
      padding: 0.75rem 1rem;
      background: var(--card-bg);
      margin-bottom: 1.75rem;
      display: flex;
      gap: 0.6rem;
      align-items: flex-start;
    }

    .textarea-icon { color: var(--muted); flex-shrink: 0; margin-top: 2px; }

    textarea {
      width: 100%;
      border: none;
      outline: none;
      resize: vertical;
      min-height: 72px;
      font-family: inherit;
      font-size: 0.95rem;
      color: var(--ink);
      background: transparent;
      line-height: 1.6;
    }

    textarea::placeholder { color: var(--rule); }
  `;

  private _textChange(e: Event) {
    this.sampleText = (e.target as HTMLTextAreaElement).value;
    this.dispatchEvent(new CustomEvent<string>(TEXT_CHANGED, {
      detail: this.sampleText,
      bubbles: true,
      composed: true,
    }));
  }

  render() {
    return html`
      <div class="textarea-wrap">
        <svg class="textarea-icon" width="18" height="18" viewBox="0 0 24 24" fill="none"
             stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="4 7 4 4 20 4 20 7"/><line x1="9" y1="20" x2="15" y2="20"/>
          <line x1="12" y1="4" x2="12" y2="20"/>
        </svg>
        <textarea
          .value=${this.sampleText}
          @input=${this._textChange}
          placeholder="Type your own sample text here…"
        ></textarea>
      </div>
    `;
  }
}

// ─── Component: <font-arena> ──────────────────────────────────────────────────

@customElement('font-arena')
export class FontArena extends LitElement {

  @state() private fonts: FontEntry[] = freshFonts();
  @state() private pair: [string, string] = ['', ''];
  @state() private sampleText: string = DEFAULT_TEXT;

  static styles = css`
    :host {
      display: block;
      font-family: Georgia, 'Times New Roman', serif;
      --ink:     #1a1410;
      --paper:   #faf8f4;
      --rule:    #d4c9b8;
      --accent:  #b5451b;
      --muted:   #7a6e62;
      --card-bg: #ffffff;
    }

    * { box-sizing: border-box; }

    .arena {
      background: var(--card-bg);
      border: 1px solid var(--rule);
      border-radius: 4px;
      padding: 1rem;
      margin-bottom: 1.75rem;
    }

    .cards {
      display: grid;
      grid-template-columns: 1fr auto 1fr;
      gap: 0.75rem;
      align-items: stretch;
    }

    @media (max-width: 600px) {
      .cards { grid-template-columns: 1fr; }
      .vs-badge { display: none; }
    }

    .font-card {
      background: var(--paper);
      border: 2px solid transparent;
      border-radius: 4px;
      padding: 1.25rem 1.5rem;
      cursor: pointer;
      text-align: left;
      transition: border-color 0.15s ease, background 0.15s ease, box-shadow 0.15s ease;
      display: block;
      width: 100%;
    }

    .font-card:hover {
      border-color: var(--accent);
      background: #fff8f6;
      box-shadow: 0 2px 12px rgba(181,69,27,0.08);
    }

    .font-card:focus-visible {
      outline: 3px solid var(--accent);
      outline-offset: 2px;
    }

    .option-label {
      font-size: 0.7rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      color: var(--muted);
      margin-bottom: 0.75rem;
      display: block;
      font-family: system-ui, sans-serif;
      opacity: 0.6;
      transition: opacity 0.15s, color 0.15s;
    }

    .font-card:hover .option-label {
      opacity: 1;
      color: var(--accent);
    }

    .sample-headline {
      font-size: 1.15rem;
      font-weight: 700;
      line-height: 1.3;
      margin: 0 0 0.5rem;
      color: var(--ink);
    }

    .sample-body {
      font-size: 0.9rem;
      line-height: 1.7;
      color: #3d3530;
      white-space: pre-wrap;
      margin: 0 0 0.4rem;
    }

    .sample-glyphs {
      font-size: 0.75rem;
      color: var(--muted);
      margin: 0;
    }

    .vs-badge {
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .vs-inner {
      width: 32px;
      height: 32px;
      border-radius: 50%;
      background: var(--card-bg);
      border: 1px solid var(--rule);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 0.65rem;
      font-weight: 900;
      color: var(--muted);
      letter-spacing: 0;
      font-family: system-ui, sans-serif;
    }

    .skip-row {
      display: flex;
      justify-content: center;
      margin-top: 0.75rem;
    }

    .skip-btn {
      background: none;
      border: none;
      cursor: pointer;
      font-size: 0.85rem;
      color: var(--muted);
      padding: 0.4rem 0.75rem;
      border-radius: 3px;
      font-family: inherit;
      font-style: italic;
      transition: background 0.15s, color 0.15s;
    }

    .skip-btn:hover { background: #f0ebe3; color: var(--ink); }
  `;

  connectedCallback() {
    super.connectedCallback();
    this.pair = pickPair(this.fonts);
    globalThis.addEventListener(FONTS_CHANGED, this._onFontsChanged as EventListener);
    globalThis.addEventListener(TEXT_CHANGED,  this._onTextChanged  as EventListener);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    globalThis.removeEventListener(FONTS_CHANGED, this._onFontsChanged as EventListener);
    globalThis.removeEventListener(TEXT_CHANGED,  this._onTextChanged  as EventListener);
  }

  private _onFontsChanged = (e: CustomEvent<FontEntry[]>) => {
    this.fonts = e.detail;
    this.pair  = pickPair(this.fonts);
  };

  private _onTextChanged = (e: CustomEvent<string>) => {
    this.sampleText = e.detail;
  };

  private _dispatch() {
    this.dispatchEvent(new CustomEvent<FontEntry[]>(FONTS_CHANGED, {
      detail: this.fonts,
      bubbles: true,
      composed: true,
    }));
  }

  private _choose(winnerId: string) {
    const loserId = this.pair.find(id => id !== winnerId)!;
    const fonts   = [...this.fonts];
    const wi = fonts.findIndex(f => f.id === winnerId);
    const li = fonts.findIndex(f => f.id === loserId);
    const { winnerElo, loserElo } = calcElo(fonts[wi], fonts[li]);
    fonts[wi] = { ...fonts[wi], elo: winnerElo, matches: fonts[wi].matches + 1 };
    fonts[li] = { ...fonts[li], elo: loserElo,  matches: fonts[li].matches + 1 };
    this.fonts = fonts;
    this.pair  = pickPair(this.fonts);
    this._dispatch();
  }

  private _skip() {
    this.pair = pickPair(this.fonts);
  }

  private _renderCard(fontId: string, label: string) {
    const font = this.fonts.find(f => f.id === fontId);
    if (!font) return html``;
    const text = this.sampleText || DEFAULT_TEXT;
    return html`
      <button class="font-card" @click=${() => this._choose(fontId)} aria-label="Choose ${font.name}">
        <span class="option-label">${label}</span>
        <div style="font-family: ${font.css}; -webkit-font-smoothing: antialiased;">
          <p class="sample-headline">Sample Headline</p>
          <p class="sample-body">${text}</p>
          <p class="sample-glyphs">1234567890 · !@#$%^&amp;*()</p>
        </div>
      </button>
    `;
  }

  render() {
    const [id1, id2] = this.pair;
    return html`
      <div class="arena">
        <div class="cards">
          ${this._renderCard(id1, 'Option A')}
          <div class="vs-badge"><div class="vs-inner">VS</div></div>
          ${this._renderCard(id2, 'Option B')}
        </div>
        <div class="skip-row">
          <button class="skip-btn" @click=${this._skip}>
            I can't decide - skip this pair
          </button>
        </div>
      </div>
    `;
  }
}

// ─── Component: <font-showdown> ───────────────────────────────────────────────

@customElement('font-showdown')
export class FontShowdown extends LitElement {
  static styles = css`
    :host { display: block; }
    .inner { max-width: 900px; margin-inline: auto; }
  `;

  render() {
    return html`
      <div class="inner">
        <font-header></font-header>
        <font-textbox></font-textbox>
        <font-arena></font-arena>
        <font-leaderboard></font-leaderboard>
      </div>
    `;
  }
}

// ─── Component: <font-leaderboard> ────────────────────────────────────────────

@customElement('font-leaderboard')
export class FontLeaderboard extends LitElement {

  @state() private fonts: FontEntry[] = freshFonts();

  static styles = css`
    :host {
      display: block;
      font-family: Georgia, 'Times New Roman', serif;
      --ink:    #1a1410;
      --paper:  #faf8f4;
      --rule:   #d4c9b8;
      --accent: #b5451b;
      --muted:  #7a6e62;
      --gold:   #c9920a;
      --silver: #6e7280;
      --bronze: #9a5c2e;
    }

    * { box-sizing: border-box; }

    .board {
      background: #ffffff;
      border: 1px solid var(--rule);
      border-radius: 4px;
      overflow: hidden;
      margin-bottom: 3rem;
    }

    .board-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1rem 1.5rem;
      background: var(--paper);
      border-bottom: 1px solid var(--rule);
    }

    .board-title {
      display: flex;
      align-items: center;
      gap: 0.6rem;
      font-size: 1.1rem;
      font-weight: 700;
      color: var(--ink);
      margin: 0;
    }

    .trophy-icon { color: var(--gold); }

    .board-meta {
      display: flex;
      align-items: center;
      gap: 1rem;
    }

    .match-count {
      font-size: 0.82rem;
      color: var(--muted);
      font-style: italic;
      font-family: system-ui, sans-serif;
    }

    .reset-btn {
      background: none;
      border: 1px solid #e0c9c9;
      color: #b03030;
      cursor: pointer;
      font-size: 0.8rem;
      padding: 0.3rem 0.75rem;
      border-radius: 3px;
      display: flex;
      align-items: center;
      gap: 0.35rem;
      font-family: system-ui, sans-serif;
      transition: background 0.15s, color 0.15s;
    }

    .reset-btn:hover { background: #fdf0f0; color: #8b1c1c; }

    table {
      width: 100%;
      border-collapse: collapse;
      table-layout: fixed;
    }

    th:nth-child(1), td:nth-child(1) { width: 4rem; }
    th:nth-child(3), td:nth-child(3) { width: 6rem; }
    th:nth-child(4), td:nth-child(4) { width: 6rem; }

    thead tr { background: var(--paper); }

    th {
      padding: 0.7rem 1.25rem;
      font-size: 0.68rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.1em;
      color: var(--muted);
      text-align: left;
      font-family: system-ui, sans-serif;
      border-bottom: 1px solid var(--rule);
      white-space: nowrap;
    }

    th.right { text-align: right; }

    tbody tr {
      border-bottom: 1px solid #f0ebe3;
      transition: background 0.12s;
    }

    tbody tr:last-child { border-bottom: none; }
    tbody tr:hover { background: #fdf9f6; }

    td {
      padding: 0.85rem 1.25rem;
      vertical-align: middle;
    }

    .rank {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 30px;
      height: 30px;
      border-radius: 50%;
      font-size: 0.78rem;
      font-weight: 800;
      font-family: system-ui, sans-serif;
    }

    .rank-1 { background: #fdf5d8; color: var(--gold); }
    .rank-2 { background: #f0f0f3; color: var(--silver); }
    .rank-3 { background: #fdf0e4; color: var(--bronze); }
    .rank-n { color: #bbb; }

    .font-name {
      font-weight: 700;
      color: var(--ink);
      font-size: 1rem;
      display: block;
    }

    .font-stack {
      font-size: 0.7rem;
      color: var(--muted);
      font-family: 'Courier New', monospace;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      display: block;
      margin-top: 2px;
    }

    .elo-pill {
      background: #f0ebe3;
      color: var(--ink);
      font-family: 'Courier New', monospace;
      font-weight: 700;
      font-size: 0.88rem;
      padding: 0.2rem 0.55rem;
      border-radius: 3px;
    }

    .td-right { text-align: right; }

    .matches-val {
      color: var(--muted);
      font-family: system-ui, sans-serif;
      font-size: 0.88rem;
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    globalThis.addEventListener(FONTS_CHANGED, this._onFontsChanged as EventListener);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    globalThis.removeEventListener(FONTS_CHANGED, this._onFontsChanged as EventListener);
  }

  private _onFontsChanged = (e: CustomEvent<FontEntry[]>) => {
    this.fonts = e.detail;
  };

  private _reset() {
    if (!confirm('Reset all rankings to zero?')) return;
    const reset = freshFonts();
    this.fonts  = reset;
    this.dispatchEvent(new CustomEvent<FontEntry[]>(FONTS_CHANGED, {
      detail: reset,
      bubbles: true,
      composed: true,
    }));
  }

  get _sorted() {
    return [...this.fonts].sort((a, b) => b.elo - a.elo);
  }

  get _totalMatches() {
    return this.fonts.reduce((s, f) => s + f.matches, 0) / 2;
  }

  private _rankClass(i: number) {
    if (i === 0) return 'rank rank-1';
    if (i === 1) return 'rank rank-2';
    if (i === 2) return 'rank rank-3';
    return 'rank rank-n';
  }

  render() {
    const sorted = this._sorted;
    return html`
      <div class="board">
        <div class="board-header">
          <h2 class="board-title">
            <svg class="trophy-icon" width="20" height="20" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6"/><path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18"/>
              <path d="M4 22h16"/><path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22"/>
              <path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22"/>
              <path d="M18 2H6v7a6 6 0 0 0 12 0V2Z"/>
            </svg>
            Leaderboard
          </h2>
          <div class="board-meta">
            <span class="match-count">${this._totalMatches} matches played</span>
            <button class="reset-btn" @click=${this._reset}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                   stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                <path d="M3 3v5h5"/>
              </svg>
              Reset
            </button>
          </div>
        </div>

        <div>
          <table>
            <thead>
              <tr>
                <th>Rank</th>
                <th>Font Stack</th>
                <th class="right">Rating</th>
                <th class="right">Matches</th>
              </tr>
            </thead>
            <tbody>
              ${repeat(sorted, f => f.id, (font, i) => html`
                <tr>
                  <td>
                    <span class="${this._rankClass(i)}">#${i + 1}</span>
                  </td>
                  <td>
                    <span class="font-name" style="font-family: ${font.css}">${font.name}</span>
                    <span class="font-stack" title="${font.css}">${font.css}</span>
                  </td>
                  <td class="td-right">
                    <span class="elo-pill">${font.elo}</span>
                  </td>
                  <td class="td-right">
                    <span class="matches-val">${font.matches}</span>
                  </td>
                </tr>
              `)}
            </tbody>
          </table>
        </div>
      </div>
    `;
  }
}
