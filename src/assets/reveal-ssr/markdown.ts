import type { Node, Parent } from 'unist';
import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkGfm from 'remark-gfm';
import remarkRehype from "remark-rehype";
import rehypeRaw from "rehype-raw";
import rehypeStringify from "rehype-stringify";
import { visit } from "unist-util-visit";


interface CodeNode extends Node {
  type: 'code';
  lang?: string;
  meta?: string;
  value: string;
}

const ESCAPED_CHARS: {[key: string]: string} = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#39;'
};

const REGEX_HL_LINES = /\[([\s\d,|-]*)\]/;
function transformCode(node: CodeNode, index: number, parent: Parent) {
  if (!node.meta || !REGEX_HL_LINES.test(node.meta)) return;
  
  const langtag = node.lang ? ` class="${node.lang}" ` : ''
  const numbers = node.meta.match(REGEX_HL_LINES)![1];
  const escaped = node.value.replace(/[&<>"']/g, match => ESCAPED_CHARS[match] || '');
  parent.children[index] = {
    type: 'html',
    value: `<pre><code data-line-numbers="${numbers}"${langtag}>${escaped}</code></pre>`,
  } as any;
}

function codePassthrough() {
  return (tree: Node, _: any) => {
    visit(tree, 'code', transformCode);
  }
}

const compiler = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(codePassthrough)
  .use(remarkRehype, {allowDangerousHtml: true})
  .use(rehypeRaw)
  .use(rehypeStringify);


const SPLIT_H = /\n-----\n/;
const SPLIT_V = /\n---\n/;


function wrapSection(content: string): string {
  return `<section>${content}</section>`;
}

export function compile(text: string): string {
  return text
    .split(SPLIT_H)
    .map(stacks => stacks.split(SPLIT_V).map(slide => String(compiler.processSync(slide))))
    .map(stack => (stack.length > 1)
      ? wrapSection(stack.map(wrapSection).join(''))
      : wrapSection(stack[0]))
    .join('');
}
