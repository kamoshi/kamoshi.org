import * as treesitter from '../../tools/treesitter/dist/index.js';
import { visit } from 'unist-util-visit';


function text(value: string) {
  return {
    type: 'text',
    value,
  }
}

function span(classes: string[], value: string) {
  return {
    type: 'element',
    tagName: 'span',
    properties: {
      className: classes.map(c => c.replace('.', '-')).join(' '),
    },
    children: [
      text(value),
    ]
  }
}


export default function rehypeTreesitter() {
  return function (tree: any) {
    visit(tree, null, (node, _, parent) => {
      if (node.tagName !== 'code' || parent.tagName !== 'pre') return;
      parent.properties.className = ['kanagawa'];

      const code = node.children?.[0].value;
      const lang = node.properties.className?.[0].replace('language-', '');
      if (!lang || !code) return;

      const stack: string[] = [];
      const children = (node.children = [] as any[] );
      const events = treesitter.hl(lang, code);

      for (const event of events) {
        switch (event.kind) {
          case 'text': {
            const child = (stack.length)
              ? span(stack, event.text)
              : text(event.text);
            children.push(child);
          } break;
          case 'open': {
            stack.push(event.name);
          } break;
          case 'close': {
            stack.pop();
          } break;
        }
      }
    });
  };
}

