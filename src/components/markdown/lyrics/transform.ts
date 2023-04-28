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
    const tag = node.rawTagName;

    // Change language context
    if (tag.match(/h\d/))
      return node.text;
    
    // Parse list as verse
    if (tag === 'ul') {
      createVerse(stack, lang);
      const lines = extractLines(node.childNodes);
      stack[lang].at(-1)!.push(...lines);
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

export function transform(html: string) {
  return reduceStack(toStack(html));
}
