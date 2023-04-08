import { defineConfig } from 'astro/config';
import rehypeKatex from 'rehype-katex';
import remarkMath from 'remark-math';
import remarkEmoji from 'remark-emoji';

// https://astro.build/config
export default defineConfig({
  markdown: {
    remarkPlugins: [
      [remarkEmoji as any, {accessible: true}],
      remarkMath,
    ],
    rehypePlugins: [
      [rehypeKatex, {output: 'mathml'}]
    ],
    shikiConfig: {
      theme: 'min-light'
    }
  }
});
