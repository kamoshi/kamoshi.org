import type { Root } from "remark-directive";
import type { Plugin } from "unified";
import { CONTINUE, SKIP, visit } from "unist-util-visit";
import { toString } from "mdast-util-to-string";


interface Options {
  sep: string;
}

type Pair = [string, string];


function toHtml([text, help]: Pair) {
  return `<ruby>${text}<rp>(</rp><rt>${help}</rt><rp>)</rp></ruby>`;
}

function createRuby(text: string, help: string, options?: Options) {
  const splitText = text.split('');
  const splitHelp = help.split(options?.sep || ';');

  const pairs = (splitText.length === splitHelp.length)
    ? splitText.map((e, i) => [e, splitHelp[i]] as Pair)
    : [[text, help]] as Pair[];

  return pairs.map(toHtml).join('');
}


export default function remarkRuby(options?: Options) {
  return (tree: any) => {
    visit(tree, "textDirective", (node, index, parent) => {
      if (node.name !== 'ruby')
        return CONTINUE;

      const text: string = toString(node);
      const help: string = node.attributes?.help;
      const ruby = {
        type: "html",
        value: createRuby(text, help, options),
        position: node.position,
      };

      parent.children.splice(index, 1, ruby);
      return SKIP;
    })
  }
}
