import { javascript } from '@codemirror/lang-javascript';
import { StreamLanguage } from '@codemirror/language';
import { lua } from '@codemirror/legacy-modes/mode/lua';
import { Compartment, EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { minimalSetup } from 'codemirror';
import { css, html, LitElement } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { lumeHighlight } from './x-editor.ts';

export type Tab = 'js' | 'lua';
export type Output =
  | { kind: 'ok'; code: string }
  | { kind: 'err'; message: string };

@customElement('x-output')
export class XOutput extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;

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

    .tabs {
      display: flex;
      border-bottom: 1px solid var(--so-border);
      flex-shrink: 0;
      background-color: var(--so-bg);
    }

    :host(.dark) .tabs {
      background-color: var(--kngw-sumiInk1);
      border-bottom-color: var(--kngw-sumiInk3);
    }

    .tab {
      padding: 0.3rem 1rem;
      font-size: 0.8rem;
      cursor: pointer;
      border: none;
      background: none;
      color: var(--c-text, inherit);
      border-bottom: 2px solid transparent;
      margin-bottom: -1px;
    }

    .tab[aria-selected='true'] {
      border-bottom-color: currentColor;
      font-weight: 600;
    }

    .body {
      flex: 1;
      overflow: hidden;
      display: flex;
      flex-direction: column;
    }

    .output-view {
      flex: 1;
      overflow: hidden;
    }

    .output-view .cm-editor {
      height: 100%;
      background-color: var(--so-bg);
      color: var(--so-fg);
    }

    :host(.dark) .output-view .cm-editor {
      background-color: var(--kngw-sumiInk1);
      color: var(--kngw-fujiWhite);
    }

    .output-view .cm-scroller {
      font-family: ui-monospace, monospace;
      font-size: 0.875rem;
      line-height: 1.6;
      overflow: auto;
    }

    .error {
      flex: 1;
      margin: 0;
      padding: 0.75rem 1rem;
      font-family: ui-monospace, monospace;
      font-size: 0.875rem;
      line-height: 1.6;
      white-space: pre-wrap;
      color: #c0392b;
      overflow: auto;
    }
  `;

  @property({ attribute: false }) output: Output = { kind: 'ok', code: '' };
  @state() private tab: Tab = 'lua';

  private outputView?: EditorView;
  private langCompartment = new Compartment();
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
    const wrap = this.shadowRoot!.querySelector('.output-view') as HTMLElement;
    this.outputView = new EditorView({
      state: EditorState.create({
        doc: '',
        extensions: [
          minimalSetup,
          lumeHighlight,
          this.langCompartment.of(this.langExtension(this.tab)),
          EditorState.readOnly.of(true),
          EditorView.editable.of(false),
        ],
      }),
      parent: wrap,
      root: this.shadowRoot!,
    });
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('output') && this.output.kind === 'ok') {
      const ov = this.outputView;
      if (!ov) return;
      ov.dispatch({
        changes: { from: 0, to: ov.state.doc.length, insert: this.output.code },
      });
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.themeObserver.disconnect();
    this.outputView?.destroy();
  }

  private langExtension(tab: Tab) {
    return tab === 'js' ? javascript() : StreamLanguage.define(lua);
  }

  private selectTab(tab: Tab) {
    this.tab = tab;
    this.outputView?.dispatch({
      effects: this.langCompartment.reconfigure(this.langExtension(tab)),
    });
    this.dispatchEvent(
      new CustomEvent('tab-change', {
        detail: tab,
        bubbles: true,
        composed: true,
      }),
    );
  }

  render() {
    return html`
      <div class="tabs">
        <button class="tab" aria-selected=${this.tab === 'lua'} @click=${() => this.selectTab('lua')}>Lua</button>
        <button class="tab" aria-selected=${this.tab === 'js'}  @click=${() => this.selectTab('js')}>JS</button>
      </div>
      <div class="body">
        <div class="output-view" ?hidden=${this.output.kind === 'err'}></div>
        ${
          this.output.kind === 'err'
            ? html`<pre class="error">${this.output.message}</pre>`
            : ''
        }
      </div>
    `;
  }
}
