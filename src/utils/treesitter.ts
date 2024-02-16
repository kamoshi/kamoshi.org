import * as treesitter from '../../tools/treesitter/dist/index.js';
import { visit } from 'unist-util-visit';


function text(value: string) {
  return {
    type: 'text',
    value,
  }
}

function span(name: string, children: any[] = []) {
  return {
    type: 'element',
    tagName: 'span',
    properties: {
      className: name.replace('.', '-'),
    },
    children,
  }
}


function highlight(lang: string, code: string) {
  const root: any[] = [];
  const ptrs: any[] = [{ children: root }];

  for (const event of treesitter.hl(lang, code)) {
    switch (event.kind) {
      case 'text': {
        const node = text(event.text);
        ptrs.at(-1).children.push(node);
      } break;
      case 'open': {
        const node = span(event.name);
        ptrs.at(-1).children.push(node);
        ptrs.push(node);
      } break;
      case 'close': {
        ptrs.pop();
      } break;
    }
  }

  return root;
}

function repl(prompt: string, lang: string, code: string) {
  const chunks = [{ i: [] as any[], o: [] as any[] }];

  for (const line of code.split('\n')) {
    if (line.startsWith(prompt)) {
      chunks.push({ i: [line], o: [] });
    } else {
      chunks.at(-1)!.o.push(line);
    }
  }

  const out: any[] = [];
  for (const { i, o } of chunks) {
    if (i.length) {
      out.push({
        type: 'element',
        tagName: 'div',
        children: [
          span('keyword-return', [text(prompt)]),
          ...highlight(lang, i[0].replace(prompt, '')),
        ]
      });
    }
    if (o.length) {
      out.push(text(o.join('\n')));
    }
  }

  return out;
}


export default function rehypeTreesitter() {
  return function (tree: any) {
    visit(tree, null, (node, _, above) => {
      if (node.tagName !== 'code' || above.tagName !== 'pre') return;
      const code = node.children?.[0].value || '';
      const lang = node.properties.className?.[0].replace('language-', '') || '';
      const parent = { ...above };

      above.tagName = 'figure';
      above.children = [parent];
      above.properties = {
        className: 'listing kanagawa',
        ...!!lang && { "data-lang": lang },
      };

      switch (lang) {
        case 'node': node.children = repl('>', 'js', code); break;
        default:     node.children = highlight(lang, code);
      }
    });
  };
}

