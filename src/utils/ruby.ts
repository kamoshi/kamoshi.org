import type { Node, Parent } from 'unist';
import { visit } from "unist-util-visit";


interface Ruby {
  text: string;
  ruby: string;
}

type Annotated =
  | string
  | Ruby;


export function transform(input: string): Annotated[] {
  const regex = /\[([^\]]+)\]\{([^}]+)\}/g;
  const parts: Annotated[] = [];
  let lastIndex = 0;

  while (true) {
    const match = regex.exec(input);
    if (!match) break;

    const [full, text, ruby] = match;

    if (match.index > lastIndex)
      parts.push(input.slice(lastIndex, match.index));

    parts.push({ text: text, ruby: ruby });
    lastIndex = regex.lastIndex;
  }

  if (lastIndex < input.length)
    parts.push(input.slice(lastIndex));

  return parts;
}

export default function ruby() {
  return (tree: Node, _: any) => {
    visit(tree, "text", (node: { value: string }, index: number, parent: Parent) => {
      const items = transform(node.value)
        .map(a => typeof a === 'object'
          ? ({ type: 'html', value: `<ruby>${a.text}<rp>(</rp><rt>${a.ruby}</rt><rp>)</rp></ruby>` })
          : ({ type: 'text', value: a })
        );

      parent.children.splice(index, 1, ...items);
    });
  }
}
