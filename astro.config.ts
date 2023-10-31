import { defineConfig } from 'astro/config';
import mdx              from '@astrojs/mdx';
import solid            from '@astrojs/solid-js';
import pagefind         from 'astro-pagefind';
import remarkDirective  from 'remark-directive';
import remarkMath       from 'remark-math';
import rehypeKatex      from 'rehype-katex';
import remarkEmoji      from 'remark-emoji';
import remarkBib        from './src/utils/remark/bib';
import remarkRuby       from './src/utils/remark/ruby';


// https://astro.build/config
export default defineConfig({
  site: 'https://kamoshi.org',
  trailingSlash: 'always',
  markdown: {
    remarkPlugins: [
      // NOTE: pinned ^2.0.0
      remarkDirective,
      // NOTE: pinned ^5.0.0
      remarkMath,
      [remarkEmoji, {accessible: true}],
      [remarkRuby, {sep: ';'}],
      remarkBib,
    ],
    rehypePlugins: [
      // https://katex.org/docs/options.html
      [rehypeKatex, {output: 'mathml'}],
    ],
    shikiConfig: {
      theme: 'min-light'
    },
  },
  experimental: {
    devOverlay: true,
  },
  integrations: [
    mdx(),
    solid(),
    pagefind(),
  ]
});
