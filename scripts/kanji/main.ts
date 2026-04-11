type Ruby = Array<[string, string]>;

export interface KKLCEntry {
  id: number;
  char: string;
  keys: string[];
  senses: string[];
  onyomi: string[];
  kunyomi: string[];
  examples: Array<[string, Ruby]>;
}

async function chooseId(): Promise<number> {
  const date = new Date().toLocaleDateString('en');
  const data = new TextEncoder().encode(date);

  const hash = await crypto.subtle.digest('SHA-256', data);

  const hashArray = Array.from(new Uint8Array(hash));
  const hashValue = hashArray.reduce((acc, byte) => acc + byte, 0);

  const min = 1;
  const max = 2300;
  return min + (hashValue % (max - min + 1));
}

function tryGetCache(id: number) {
  const item = localStorage.getItem('kanji');
  if (!item) return;

  const cache = JSON.parse(item);
  if (cache.id === id) {
    return cache.data;
  }
}

function insertCache(id: number, data: KKLCEntry) {
  localStorage.setItem('kanji', JSON.stringify({ id, data }));
}

export async function getKanji(): Promise<KKLCEntry> {
  const id = await chooseId();

  const cache = tryGetCache(id);
  if (cache) return cache;

  const data = await fetch(`/static/kanji/${id}.json`).then((res) =>
    res.json(),
  );
  insertCache(id, data);
  return data;
}

async function getKanjiSVG(char: string): Promise<string> {
  const codePoint = char.codePointAt(0);

  if (codePoint === undefined) {
    throw new Error(`Invalid character input: ${char}`);
  }

  const hexCode = codePoint.toString(16).toLowerCase().padStart(5, '0');
  const url = `/static/svg/kanji/${hexCode}.svg`;

  const data = await fetch(url);
  const text = await data.text();

  return text
    .substring(text.indexOf('<svg'))
    .replaceAll(/<g id="kvg:StrokeNumbers[\s\S]*?<\/g>/g, '')
    .replaceAll('stroke:#000000', 'stroke:currentColor');
}

class KanjiOfDay extends HTMLElement {
  connectedCallback() {
    this.load();
  }

  private async load() {
    const state = await getKanji();
    const svg = await getKanjiSVG(state.char);
    this.renderKanji(state, svg);
  }

  private renderKanji(state: KKLCEntry, svg: string) {
    const examples = state.examples
      .map(([meaning, example]) => {
        const ruby = example
          .map(([expr, r]) => `${expr}<rt>${r || ''}</rt>`)
          .join('');
        return `<div class="cell-ja"><ruby>${ruby}</ruby></div><div class="cell-en">${meaning}</div>`;
      })
      .join('');

    this.innerHTML = `
      <h2 class="p-card__heading">Kanji of the day</h2>
      <div class="daily-kanji">
        <div class="info">
          <div class="info-box">
            <div class="info-id">#${state.id}</div>
            ${svg}
          </div>
          <div class="info-meta">
            <div class="info-key">${state.keys.join(', ')}</div>
            <div class="info-on">${state.onyomi.join(', ')}</div>
            <div class="info-kun">${state.kunyomi.join(', ')}</div>
          </div>
        </div>
        <section class="table">${examples}</section>
      </div>`;
  }
}

customElements.define('kanji-of-day', KanjiOfDay);
