import { defineConfig } from 'astro/config';
import rehypeKatex from 'rehype-katex';
import remarkMath from 'remark-math';
import remarkEmoji from 'remark-emoji';
import svelte from "@astrojs/svelte";


// https://astro.build/config
export default defineConfig({
  site: 'https://kamoshi.org',
  trailingSlash: 'always',
  markdown: {
    remarkPlugins: [
      [(remarkEmoji as any), {accessible: true}],
      remarkMath
    ],
    rehypePlugins: [
      [rehypeKatex, {output: 'mathml'}]
    ],
    shikiConfig: {
      theme: 'min-light'
    }
  },
  integrations: [svelte()]
});
