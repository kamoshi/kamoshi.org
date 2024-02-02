export interface KKLCEntry {
  entry: number;
  char: string;
  meanings: string[];
  on: string[];
  kun: string[];
  examples: Array<
    [Array<[string, string]>, string]
  >;
  senses: string[];
}


async function chooseKanji(): Promise<number> {
  const date = new Date().toLocaleDateString('en');
  console.log(date);
  const data = new TextEncoder().encode(date);

  const hash = await crypto.subtle.digest('SHA-256', data);

  const hashArray = Array.from(new Uint8Array(hash));
  const hashValue = hashArray.reduce((acc, byte) => acc + byte, 0);

  const min = 1;
  const max = 2300;
  const id = min + (hashValue % (max - min + 1));
  return id;
}

export async function fetchKanji(): Promise<KKLCEntry> {
  return await fetch(`/static/kanji/${await chooseKanji()}.json`).then(res => res.json());
}
