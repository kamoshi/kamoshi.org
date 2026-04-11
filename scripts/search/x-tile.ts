import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { unsafeHTML } from 'lit/directives/unsafe-html.js';
import type { PagefindDocument } from './pagefind.ts';

@customElement('x-tile')
export class XTile extends LitElement {

  @property({ type: Object })
  data?: PagefindDocument;

  static styles = css`
    a {
      display: block;
      padding: 0.5em;
      background-color: var(--c-bg-paper);
      box-shadow:
        rgba(0, 0, 0, 0.1) 0px 1px 3px 0px,
        rgba(0, 0, 0, 0.06) 0px 1px 2px 0px;
      transition: box-shadow linear 100ms;
      text-decoration: none;
      color: var(--c-text);
    }

    a:focus-within,
    a:hover {
      box-shadow:
        rgba(0, 0, 0, 0.1) 0px 4px 6px -1px,
        rgba(0, 0, 0, 0.06) 0px 2px 4px -1px;
    }

    header {
      display: flex;
    }

    h2 {
      font-size: 1.3em;
      margin: 0;
    }

    .date {
      margin-left: auto;
    }

    .excerpt mark {
      background-color: unset;
      font-weight: 800;
      text-decoration: underline;
      color: var(--c-secondary);
    }
  `;

  render() {
    if (!this.data) return html``;

    return html`
      <a href=${this.data.url}>
        <header>
          <h2>${this.data.meta?.title || 'Untitled'}</h2>
        </header>
        <div class="excerpt">
          ${unsafeHTML(this.data.excerpt)}
        </div>
      </a>
    `;
  }
}
