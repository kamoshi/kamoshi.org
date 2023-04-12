import type { Node, Parent } from 'unist';
import { visit } from "unist-util-visit";


interface RubyNode extends Node {
  type: 'html';
  value: string;
}

const regex = /\{.+?\}\(.+?\)/g;
const group = /^\{(.+?)\}\((.+?)\)$/;
const template = "<ruby>$1<rp>(</rp><rt>$2</rt><rp>)</rp></ruby>";

function toRuby(ruby: string): RubyNode {
  return {
    type: "html",
    value: ruby.replace(group, template),
  }
}

function transformRuby(node: {value: string}, index: number, parent: Parent) {
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
