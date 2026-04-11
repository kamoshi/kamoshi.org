import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { repeat } from 'lit/directives/repeat.js';
import type { Pagefind, PagefindDocument } from './pagefind.ts';
import './x-tile.ts';

const PAGEFIND_URL = '/_pagefind/pagefind.js';

interface PagefindResult {
  id: string;
  data(): Promise<PagefindDocument>;
}

@customElement('x-search')
export class XSearch extends LitElement {
  @state() private client?: Pagefind;
  @state() private query = '';
  @state() private limit = 10;
  @state() private results: PagefindDocument[] = [];
  @state() private searchResults: PagefindResult[] = [];
  @state() private loading = false;

  private debounceTimer?: number;
  private throttleTime = 0;
  private scrollHandler = this.onScroll.bind(this);
  private popstateHandler = this.sync.bind(this);

  static styles = css`
    article {
      max-width: 52em;
      margin: 0.5em auto;
      padding: 0 1em;

      @media (min-width: 40em) {
        margin: 2em auto;
        padding: 0 4em;
      }
    }

    input {
      width: 100%;
      padding: 0.5em 1em;
      margin-bottom: 0.5em;
      box-sizing: border-box;
      background-color: var(--c-bg-paper);
      color: var(--c-text);
      border: 1px solid var(--c-border);
    }

    section {
      display: grid;
      row-gap: 0.5em;
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    globalThis.addEventListener('scroll', this.scrollHandler);
    globalThis.addEventListener('popstate', this.popstateHandler);
    this.sync();
    import(PAGEFIND_URL).then((pf) => {
      this.client = pf;
      this.search();
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    globalThis.removeEventListener('scroll', this.scrollHandler);
    globalThis.removeEventListener('popstate', this.popstateHandler);
  }

  private sync() {
    const q = new URLSearchParams(globalThis.location.search).get('q') || '';
    if (q !== this.query) {
      this.query = q;
      this.limit = 10;
      this.search();
    }
  }

  private async search() {
    if (!this.client || !this.query) {
      this.searchResults = [];
      this.results = [];
      return;
    }

    this.loading = true;
    const { results } = await this.client.search(this.query);
    this.searchResults = results;
    this.results = [];
    await this.loadMore();
    this.loading = false;
  }

  private async loadMore() {
    const needed = Math.min(this.limit, this.searchResults.length);
    if (this.results.length >= needed) return;

    const newPages = await Promise.all(
      this.searchResults.slice(this.results.length, needed).map((r) => r.data())
    );
    this.results = [...this.results, ...newPages];
  }

  private onInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    this.query = value;
    this.limit = 10;
    this.search();

    clearTimeout(this.debounceTimer);
    const url = new URL(globalThis.location.href);
    value ? url.searchParams.set('q', value) : url.searchParams.delete('q');
    this.debounceTimer = globalThis.setTimeout(
      () => globalThis.history.pushState({}, '', url),
      1000
    );
  }

  private onScroll() {
    const now = Date.now();
    if (now - this.throttleTime < 200) return;

    const { scrollHeight } = document.documentElement;
    const { innerHeight, scrollY } = window;
    if (scrollHeight - (innerHeight + scrollY) < 100) {
      this.throttleTime = now;
      this.limit += 5;
      this.loadMore();
    }
  }

  render() {
    return html`
      <article>
        <h1>Search</h1>
        <input
          placeholder="Start typing here!"
          .value=${this.query}
          @input=${this.onInput}
        />
        ${this.query
          ? this.loading
            ? html`Loading...`
            : html`
                <section>
                  <div>Showing results for "${this.query}" (${this.searchResults.length})</div>
                  ${repeat(
                    this.results,
                    (page) => page.url,
                    (page) => html`<x-tile .data=${page}></x-tile>`
                  )}
                </section>
              `
          : html`<div>No results to show yet...</div>`}
      </article>
    `;
  }
}
