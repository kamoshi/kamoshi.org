interface Stack {
  [key: string]: string[][];
}

interface Verse {
  [key: string]: string[];
}


function increaseStack(data: Stack, lang: string) {
  lang in data
    ? data[lang].push([])
    : data[lang] = [[]];
}

function fromMarkdown(markdown: string): Stack {
  const stack: Stack = {};
  if (!markdown) return stack;

  let space = true;
  let lang = '';
  for (const line of markdown.split('\n').map(x => x.trim())) {
    if (line.startsWith('#')) {
      lang = line.match(/#+ (.+)/)![1];
      space = true;
    }
      
    if (line === '' || line.startsWith('---'))
      space = true;
    
    if (line.startsWith('- ')) {
      if (space === true) {
        increaseStack(stack, lang);
        space = false;
      }

      const text = line.match(/- (.+)/)![1];
      stack[lang].at(-1)!.push(text);
    }
  }

  return stack
}

function reduceStack(stack: Stack): Verse[] {
  const langs = Object.keys(stack);
  const length = langs.reduce((acc, lang) => Math.max(acc, stack[lang].length), 0);
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

export function transform(data: string, markdown = false) {
  return reduceStack(fromMarkdown(data));
}
