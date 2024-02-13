import * as treesitter from '../../tools/treesitter/dist/index.js';
import { visit } from 'unist-util-visit';


function text(value: string) {
  return {
    type: 'text',
    value,
  }
}

function span(name: string) {
  return {
    type: 'element',
    tagName: 'span',
    properties: {
      className: name.replace('.', '-'),
    },
    children: []
  }
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

      const root = { children: [] };
      const ptrs: any[] = [root];

      for (const event of treesitter.hl(lang, code)) {
        switch (event.kind) {
          case 'text': {
            const inserted = text(event.text);
            ptrs.at(-1).children.push(inserted);
          } break;
          case 'open': {
            const inserted = span(event.name);
            ptrs.at(-1).children.push(inserted);
            ptrs.push(inserted);
          } break;
          case 'close': {
            ptrs.pop();
          } break;
        }
      }

      node.children = root.children;
    });
  };
}

