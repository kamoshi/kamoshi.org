import { defineConfig } from 'astro/config';
import sitemap          from '@astrojs/sitemap';
import mdx              from '@astrojs/mdx';
import svelte           from '@astrojs/svelte';
import pagefind         from 'astro-pagefind';
import remarkDirective  from 'remark-directive';
import remarkMath       from 'remark-math';
import rehypeKatex      from 'rehype-katex';
import remarkEmoji      from 'remark-emoji';
import remarkBib        from './src/utils/remark/bib';
import remarkRuby       from './src/utils/remark/ruby';
import rehypeTreesitter from './src/utils/treesitter';


// https://astro.build/config
export default defineConfig({
  site: 'https://kamoshi.org',
  trailingSlash: 'always',
  markdown: {
    syntaxHighlight: false,
    remarkPlugins: [
      remarkDirective,
      remarkMath,
      [remarkEmoji, { accessible: true }],
      [remarkRuby, { sep: ';' }],
      remarkBib,
    ],
    rehypePlugins: [
      // https://katex.org/docs/options.html
      [rehypeKatex, { output: 'mathml' }],
      rehypeTreesitter,
    ],
  },
  integrations: [
    mdx(),
    svelte({ compilerOptions: { runes: true } }),
    pagefind(),
    sitemap()
  ]
});
