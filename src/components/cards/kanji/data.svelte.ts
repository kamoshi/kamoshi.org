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

  const data = await fetch(`/static/kanji/${id}.json`).then(res => res.json());
  insertCache(id, data);
  return data;
}
