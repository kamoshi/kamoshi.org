import { visit } from "unist-util-visit";
import type { Node } from "unist-util-visit/lib";


const regex = /\{.+?\}\(.+?\)/g;
const group = /^\{(.+?)\}\((.+?)\)$/;
const template = "<ruby>$1<rp>(</rp><rt>$2</rt><rp>)</rp></ruby>";

function toRuby(ruby: string) {
  return ({
    type: "html",
    value: ruby.replace(group, template),
  })
}

function transformRuby(node: { value: string }, index: number, parent: any) {
  if (!regex.test(node.value)) return;

  const text = node.value.split(regex).map(value => ({ type: "text", value}));
  const ruby = node.value.match(regex)!.map(toRuby);

  const merged = [];
  for (let i = 0; i < text.length; i++) {
    text[i] && merged.push(text[i]);
    ruby[i] && merged.push(ruby[i]);
  }

  parent.children.splice(index, 1, ...merged);
}

export default function ruby() {
  return (tree: Node, _: any) => {
    visit(tree, "text", transformRuby);
  }
}
