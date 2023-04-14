---
title: The quest to remove JavaScript by writing JavaScript
date: 2023-04-14T20:34:04.765Z
---

Moving this website to a JavaScript based SSG turned out to be a wonderful idea, because now it is actually possible to use the same code both on frontend and on the backend. It means that, where possible, I can execute code that would normally be run in the browser. For example, while using [KaTeX](https://katex.org/) I can prerender the math equations very easily, so that they are already rendered in the browser with no additional JS needed. The idea that we should be able to run the same stuff both client-side and server-side is already well established. It's known as [isomorphic JavaScript](https://en.wikipedia.org/wiki/Isomorphic_JavaScript).

Unfortunately, despite being able to run the same JS in the browser and on the server there are still pieces of code which can't work in the server. These are the libraries/modules that use the browsers API such as `window`, `localStorage`, etc. Sadly some packages simply can't be run on the server and must be delegated to the browser.


## Reveal.js

Reveal.js is an example of a really cool library that unfortunately mostly works client-side. While working on this website I decided to use it to render slides from Markdown, which would require parsing markdown into a structure of HTML understood by this library in particular. The first thing I tried is to somehow try to port the Markdown plugin for Reveal.js into server.

Once I tried to work on [the source code](https://github.com/hakimel/reveal.js/blob/master/plugin/markdown/plugin.js), I realized that it is pretty poorly written. Now I'm not here to judge people, Reveal.js is a great piece of software, however, the code has no JSDoc type annotations, and there's mutable state all over the place. The functions contained within the plugin could be extracted into the top level and made pure.

Another big problem is how the plugin uses references to HTML in a few different places. I also don't really like how the plugin includes some code for fetching data from the internet, I don't need this at all. Not only that, but I'm not sure if the way the author wrote this code allows for tree-shaking to happen. I feel like fixing these things would generally make it much easier to reason about the logic.

After seeing this code I decided to see whether it would be possible to recreate this plugin. I decided to use remark for this, as that is what Astro uses anyway.

## Markdown converter

The first thing that should happen is splitting the markdown. We can use simple Regexes.
```ts
const SPLIT_H = /\n-----\n/;
const SPLIT_V = /\n---\n/;
```

The Reveal.js library expects slides to be wrapped in `<section>` elements, so we can write a helper function that wraps anything in this HTML tag:
```ts
function wrapSection(content: string): string {
  return `<section>${content}</section>`;
}
```

Another thing we would like to do while parsing Markdown is to make sure that the parser properly converts the code blocks along with any highlighted line numbers. This is only important if we want to use this feature in the Highlight plugin.

Nodes representing code block have the following structure in Markdown AST:
```ts
interface CodeNode extends Node {
  type: 'code';
  lang?: string;
  meta?: string;
  value: string;
}
```

Now we can write a transformer function that converts `CodeNode`s into HTML that preserves highlighted line numbers. While doing this we should also escape any chars which could mess up the HTML output.
```ts
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
```

Now we can build the parser, which will use `unified` as its foundation:
```ts
const renderer = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(codePassthrough)
  .use(remarkRehype, {allowDangerousHtml: true})
  .use(rehypeRaw)
  .use(rehypeStringify);
```

And now all that is left to do is to split the input Markdown into slides using our Regexes from the earlier, and then parse each slide individually with our renderer:
```ts
export function render(text: string): string {
  return text
    .split(SPLIT_H)
    .map(stacks => stacks.split(SPLIT_V).map(slide => String(renderer.processSync(slide))))
    .map(stack => (stack.length > 1)
      ? wrapSection(stack.map(wrapSection).join(''))
      : wrapSection(stack[0]))
    .join('');
}
```

And that's it… mostly. We didn't replicate the full functionality of the Markdown plugin for Reveal.js, but we got pretty far writing so little code :smile:. And it all works server-side too!

Now we can render slides server side, and it just works:

```astro
---
const { entry } = Astro.props;
const slides = render(entry.body);
---
<div class="reveal">
  <div class="slides" set:html={slides}></div>
</div>
```

## What next?

I would like to try converting the Highlight plugin to work server-side. That plugin is huge, because it includes highlight.js inside itself. We end up sending ~1MB to the client and it's mostly just random stuff. I feel like this could be done in advance too :sweat_smile:.
