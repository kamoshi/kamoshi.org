import {
  autocompletion,
  type CompletionContext,
  type CompletionResult,
} from '@codemirror/autocomplete';
import {
  HighlightStyle,
  StreamLanguage,
  syntaxHighlighting,
} from '@codemirror/language';
import type { Diagnostic } from '@codemirror/lint';
import { linter, lintGutter } from '@codemirror/lint';
import { EditorView, hoverTooltip } from '@codemirror/view';
import { tags as t } from '@lezer/highlight';
import { basicSetup } from 'codemirror';
import { css, html, LitElement } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { wasmComplete, wasmLint, wasmReady, wasmTypeAt } from './wasm.ts';

// ── Language definition ────────────────────────────────────────────────────

const KEYWORDS = /^(let|type|use|if|then|else|in|pub|and)\b/;
const BUILTINS = /^(map|filter|fold|sum|average|sort|show|print)\b/;
const TYPES = /^(Num|Text|Bool|List|Maybe|Result)\b/;
const CTORS = /^[A-Z]\w*/;

export const lumeLang = StreamLanguage.define({
  token(stream) {
    if (stream.match('--')) {
      stream.skipToEnd();
      return 'comment';
    }
    if (stream.match(/"(?:[^"\\]|\\.)*"/)) return 'string';
    if (stream.match(/^[0-9]+(?:\.[0-9]+)?/)) return 'number';
    if (stream.match(KEYWORDS)) return 'keyword';
    if (stream.match(BUILTINS)) return 'variableName.standard';
    if (stream.match(TYPES)) return 'typeName';
    const prev = stream.string[stream.pos - 1] ?? '';
    if (!/\w/.test(prev) && stream.match(CTORS)) return 'className';
    if (stream.match(/^(?:\|>|\?>|->|\+\+|==|!=|<=|>=|[+\-*/<>=|])/))
      return 'operator';
    if (stream.match(/^[{}()[\],.:#]/)) return 'punctuation';
    stream.next();
    return null;
  },
});

export const lumeHighlight = syntaxHighlighting(
  HighlightStyle.define([
    { tag: t.comment, color: 'var(--hl-comment)', fontStyle: 'italic' },
    { tag: t.string, color: 'var(--hl-string)' },
    { tag: t.number, color: 'var(--hl-number)' },
    { tag: t.keyword, color: 'var(--hl-keyword)', fontWeight: 'bold' },
    { tag: t.standard(t.variableName), color: 'var(--hl-builtin)' },
    { tag: t.typeName, color: 'var(--hl-type)', fontWeight: 'bold' },
    { tag: t.className, color: 'var(--hl-ctor)' },
    { tag: t.operator, color: 'var(--hl-operator)' },
    { tag: t.punctuation, color: 'var(--hl-punct)' },
  ]),
);

// ── CM extensions ──────────────────────────────────────────────────────────

function lumeLinter() {
  return linter(
    (view): readonly Diagnostic[] => {
      if (!wasmReady) return [];
      const src = view.state.doc.toString();
      let raw: { from: number; to: number; message: string }[];
      try {
        raw = JSON.parse(wasmLint(src));
      } catch {
        return [];
      }
      return raw.map((d) => ({
        from: d.from,
        to: Math.max(d.to, d.from + 1),
        severity: 'error' as const,
        message: d.message,
      }));
    },
    { delay: 0 },
  );
}

function lumeHover() {
  return hoverTooltip((view, pos) => {
    if (!wasmReady) return null;
    const src = view.state.doc.toString();
    const ty = wasmTypeAt(src, pos);
    if (!ty) return null;
    return {
      pos,
      create() {
        const dom = document.createElement('div');
        dom.className = 'cm-lume-type';
        dom.textContent = ty;
        return { dom };
      },
    };
  });
}

// ── Autocomplete ──────────────────────────────────────────────────────────

/**
 * Classify the cursor position for completion purposes.
 * - `'use-path'` — inside the path string of a `use` declaration
 * - `'expr'`     — in a normal expression position
 * - `'none'`     — inside a non-use string or a line comment; suppress completions
 */
function completionCtx(
  src: string,
  offset: number,
): 'use-path' | 'expr' | 'none' {
  const lineStart = src.lastIndexOf('\n', offset - 1) + 1;
  const line = src.slice(lineStart, offset);

  let inString = false;
  let quoteIdx = -1;

  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (!inString && ch === '-' && line[i + 1] === '-') return 'none';
    if (ch === '"') {
      if (inString) {
        inString = false;
        quoteIdx = -1;
      } else {
        inString = true;
        quoteIdx = i;
      }
    } else if (inString && ch === '\\') {
      i++; // skip escaped char
    }
  }

  if (!inString) return 'expr';

  // We're inside a string — only allow completions in use-path strings.
  const beforeQuote = line.slice(0, quoteIdx).trim();
  return /^use\s/.test(beforeQuote) && beforeQuote.trimEnd().endsWith('=')
    ? 'use-path'
    : 'none';
}

function lumeCompletionSource(
  context: CompletionContext,
): CompletionResult | null {
  if (!wasmReady) return null;

  const src = context.state.doc.toString();
  const pos = context.pos;
  const ctx = completionCtx(src, pos);

  if (ctx === 'none') return null;

  if (ctx === 'use-path') {
    // The WASM complete() detects the use-path context and returns stdlib
    // names (or [] for file paths, which WASM can't resolve).
    let items: { label: string; detail: string }[];
    try {
      items = JSON.parse(wasmComplete(src, pos));
    } catch {
      return null;
    }
    if (!items.length && !context.explicit) return null;
    // Replace from the start of whatever module name fragment is typed.
    const word = context.matchBefore(/\w*/);
    return {
      from: word ? word.from : pos,
      options: items.map((item) => ({
        label: item.label,
        detail: item.detail,
        type: 'module' as const,
      })),
    };
  }

  // expr context — identifier and field-access completions.
  const word = context.matchBefore(/\w*/);
  if (!word) return null;
  const charBefore = context.state.doc.sliceString(word.from - 1, word.from);
  if (word.from === word.to && charBefore !== '.' && !context.explicit)
    return null;

  let items: { label: string; detail: string }[];
  try {
    items = JSON.parse(wasmComplete(src, word.to));
  } catch {
    return null;
  }
  if (!items.length) return null;

  return {
    from: word.from,
    options: items.map((item) => ({
      label: item.label,
      detail: item.detail,
      type: 'variable' as const,
    })),
  };
}

function lumeComplete() {
  return autocompletion({ override: [lumeCompletionSource] });
}

// ── Component ──────────────────────────────────────────────────────────────

const hlVars = css`
  :host {
    --hl-comment:  var(--so-comment);
    --hl-string:   var(--so-green);
    --hl-number:   var(--so-orange);
    --hl-keyword:  var(--so-blue);
    --hl-builtin:  var(--so-orange);
    --hl-type:     var(--so-orange);
    --hl-ctor:     var(--so-purple);
    --hl-operator: var(--so-fg);
    --hl-punct:    var(--so-gray);
  }
  :host(.dark) {
    --hl-comment:  var(--kngw-fujiGray);
    --hl-string:   var(--kngw-springGreen);
    --hl-number:   var(--kngw-sakuraPink);
    --hl-keyword:  var(--kngw-oniViolet);
    --hl-builtin:  var(--kngw-springBlue);
    --hl-type:     var(--kngw-waveAqua2);
    --hl-ctor:     var(--kngw-surimiOrange);
    --hl-operator: var(--kngw-boatYellow2);
    --hl-punct:    var(--kngw-springViolet2);
  }
`;

@customElement('x-editor')
export class XEditor extends LitElement {
  static styles = [
    hlVars,
    css`
      :host {
        display: block;
        height: 100%;
      }

      .wrap {
        height: 100%;
        overflow: hidden;
      }

      .cm-editor {
        height: 100%;
        background-color: var(--so-bg);
        color: var(--so-fg);
      }

      .cm-gutters {
        background-color: var(--so-bg);
        border-right-color: var(--so-border);
        color: var(--so-comment);
      }

      :host(.dark) .cm-editor {
        background-color: var(--kngw-sumiInk1);
        color: var(--kngw-fujiWhite);
      }

      :host(.dark) .cm-gutters {
        background-color: var(--kngw-sumiInk1);
        border-right-color: var(--kngw-sumiInk3);
        color: var(--kngw-sumiInk4);
      }

      .cm-scroller {
        font-family: ui-monospace, monospace;
        font-size: 0.875rem;
        line-height: 1.6;
        overflow: auto;
      }

      .cm-tooltip {
        background-color: var(--so-bg);
        color: var(--so-fg);
        border-color: var(--so-border);
      }

      :host(.dark) .cm-tooltip {
        background-color: var(--kngw-sumiInk3);
        color: var(--kngw-fujiWhite);
        border-color: var(--kngw-sumiInk4);
      }

      :host(.dark) .cm-tooltip-lint {
        background-color: var(--kngw-sumiInk3);
      }

      .cm-lume-type {
        padding: 2px 6px;
        font-family: ui-monospace, monospace;
        font-size: 0.8rem;
        background: var(--so-bg);
        border: 1px solid var(--so-border);
        border-radius: 3px;
      }

      :host(.dark) .cm-lume-type {
        background: var(--kngw-sumiInk3);
        border-color: var(--kngw-sumiInk4);
      }
    `,
  ];

  @property({ attribute: false }) initialDoc = '';

  private view?: EditorView;
  private themeObserver = new MutationObserver(() => this.syncDark());

  private syncDark() {
    this.classList.toggle(
      'dark',
      document.documentElement.classList.contains('dark'),
    );
  }

  connectedCallback() {
    super.connectedCallback();
    this.themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    });
    this.syncDark();
  }

  firstUpdated() {
    const wrap = this.shadowRoot!.querySelector('.wrap') as HTMLElement;
    this.view = new EditorView({
      doc: this.initialDoc,
      extensions: [
        basicSetup,
        lumeLang,
        lumeHighlight,
        lintGutter(),
        lumeLinter(),
        lumeHover(),
        lumeComplete(),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            this.dispatchEvent(
              new CustomEvent('lume-change', {
                detail: update.state.doc.toString(),
                bubbles: true,
                composed: true,
              }),
            );
          }
        }),
      ],
      parent: wrap,
      root: this.shadowRoot!,
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.themeObserver.disconnect();
    this.view?.destroy();
  }

  getDoc(): string {
    return this.view?.state.doc.toString() ?? '';
  }

  render() {
    return html`<div class="wrap"></div>`;
  }
}
