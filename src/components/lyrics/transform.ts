import { Node, HTMLElement, parse } from 'node-html-parser';


interface Stack {
  [key: string]: string[][];
}

interface Verse {
  [key: string]: string[];
}


function extractLines(nodes: Node[]): string[] {
  return nodes
    .map(node => node.text)
    .filter(text => text != '\n');
}

function createVerse(data: Stack, lang: string) {
  (lang in data)
    ? data[lang].push([])
    : data[lang] = [[]];
}

function toStack(html: string): Stack {
  const root = parse(html);
  const stack: Stack = {};

  const nodes = root.childNodes as HTMLElement[];
  nodes.reduce((lang, node) => {
    switch (node.rawTagName) {
      case 'h1': {
        const lang: string = node.id.replace(/-.+/, '');
        createVerse(stack, lang);
        return lang;
      }
      case 'ul': {
        const lines = extractLines(node.childNodes);
        stack[lang].at(-1)!.push(...lines);
        return lang;
      }
    }
      return lang;
  }, '');

  return stack;
}

function reduceStack(stack: Stack): Verse[] {
  const langs = Object.keys(stack);
  const length = langs.map(lang => stack[lang].length).reduce((a, b) => Math.max(a, b));
  const verses: Verse[] = [];
  
  for (const _ of Array(length)) {
    const verse: Verse = {};
    for (const lang of langs) {
      const lines = stack[lang].pop();
      verse[lang] = lines ? lines : [];
    }
    verses.push(verse);
  }
  return verses.reverse();
}

function toHtml(verses: Verse[]): string {
  const keys = Object.keys(verses[0]);
  const head = keys.map(lang => `<th>${lang}</th>`);
  const rows = verses.map(verse =>
    `<tr>${keys.map(lang =>
      `<td>${verse[lang].map(line =>
          `<span>${line}</span><br/>`).join('')}
        </td>`).join('')}
      </tr>`
    )
    .join('');

  return [
    "<table>",
    `<tr>${head.join('')}</tr>`,
    rows,
    "</table>",
  ].join('');
}

export function transform(html: string) {
  return toHtml(reduceStack(toStack(html)));
}
