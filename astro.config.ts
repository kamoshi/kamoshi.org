import { defineConfig } from 'astro/config';
import rehypeKatex from 'rehype-katex';
import remarkMath from 'remark-math';
import remarkEmoji from 'remark-emoji';
import remarkRuby from './src/utils/ruby';
import mdx from '@astrojs/mdx';
import solid from '@astrojs/solid-js';


// https://astro.build/config
export default defineConfig({
  site: 'https://kamoshi.org',
  trailingSlash: 'always',
  markdown: {
    remarkPlugins: [
      [(remarkEmoji as any), {accessible: true}],
      remarkMath,
      remarkRuby,
    ],
    rehypePlugins: [
      [rehypeKatex, {output: 'mathml'}]
    ],
    shikiConfig: {
      theme: 'min-light'
    }
  },
  integrations: [
    mdx(),
    solid(),
  ]
});
