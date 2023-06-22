import { defineConfig } from 'astro/config';
import rehypeKatex from 'rehype-katex';
import remarkMath from 'remark-math';
import remarkEmoji from 'remark-emoji';
import mdx from '@astrojs/mdx';
import solid from '@astrojs/solid-js';
import pagefind from 'astro-pagefind';
import remarkDirective from 'remark-directive';
import remarkBibliography from "./src/utils/remark/bibliography";
import remarkRuby from "./src/utils/remark/ruby";


// https://astro.build/config
export default defineConfig({
  site: 'https://kamoshi.org',
  trailingSlash: 'always',
  compressHTML: true,
  markdown: {
    remarkPlugins: [
      remarkDirective,
      [remarkRuby, {separator: ';'}],
      remarkBibliography,
      [remarkEmoji as any, {accessible: true}],
      remarkMath,
    ],
    rehypePlugins: [
      [rehypeKatex, {output: 'mathml'}]
    ],
    shikiConfig: {
      theme: 'min-light'
    }
  },
  experimental: {
    assets: true,
  },
  integrations: [
    mdx(),
    solid(),
    pagefind(),
  ]
});
