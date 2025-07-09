---
title: Ruby extensions for Markdown
date: 2023-01-15T16:19:00+01:00
tags: [japanese, zola, hugo, astro]
---

Sadly, as far as I know CommonMark currently doesn't include anything about ruby in its spec. On top of that ruby is pretty uncommon, so it is pretty rare for any ruby extensions to exist. As I move through any new frameworks, I will try to document any simple solutions that I figure out.

## Examples

| Language    | Example |
| --------    | ------- |
| Japanese    | [日本語]{にほんご}の[文法]{ぶんぽう}は[難]{むずか}しい |
| Chinese     | [北京]{Běijīng}<br/>[北京]{ㄅㄟˇㄐㄧㄥ} |
| Korean      | [韓國]{한국} |
| Vietnamese  | [河內]{HàNội} |
| Other       | I [love]{like} ruby! |


## Remark
Recently I moved to Astro, which generally uses JavaScript tools for parsing and manipulating markdown. In particular that's the Remark and Rehype from Unified.

When looking for a way to extend remark I first looked for an existing plugin which would allow me to automatically convert custom shorthands for ruby inserted into Markdown. I found a plugin called [remark-ruby](https://github.com/laysent/remark-ruby#readme), but honestly after looking at its source code I decided to hand roll my own solution. It just looked overcomplicated for something that should be simple and easy to modify (for me).

I was able to write a really simple and short solution using a pair of Regexes working in conjunction to split strings and replace custom ruby shorthands with HTML, which then passes through to Rehype.

```typescript
import { visit } from "unist-util-visit";
import type { Node } from "unist-util-visit/lib";


const regex = /\{.+?\}\(.+?\)/g;
const group = /^\{(.+?)\}\((.+?)\)$/;
const template = "<ruby>$1<rp>(</rp><rt>$2</rt><rp>)</rp></ruby>";

function toRuby(ruby: string) {
  return ({
    type: "html",
    value: ruby.replace(group, template),
  })
}

function transformRuby(node: { value: string }, index: number, parent: any) {
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
```

Usage:
`{日本語}(にほんご)の{文法}(ぶんぽう)は{難}(むずか)しい`

## Zola
The following is a snippet for the Tera templating engine which is inspired by Jinja2.

```html
<ruby>
  {%- for item in expr | split(pat=";") -%}
  {%- set sub_item = item | split(pat=",") -%}
  {{- sub_item[0] -}}
  {%- if sub_item[1] -%}
  <rp>(</rp><rt>{{- sub_item[1] -}}</rt><rp>)</rp>
  {%- else -%}
  <rt></rt>
  {%- endif -%}
  {%- endfor -%}
</ruby>
```

Usage:
`{{ ruby(expr="日本語,にほんご;の;文法,ぶんぽう;は;難,むずか;しい") }}`

## Hugo
The following is a snippet for the Golang templating engine used by Hugo.
```html
{{- with .Get 0 -}}
<ruby>
  {{- /* Generate the ruby markup */ -}}
  {{- range split . ";" -}}
    {{- $item := split . "," -}}
    {{- $ruby := index $item 1 -}}
    {{- index $item 0 -}}
    {{- if $ruby -}}
      <rp>(</rp><rt>{{- $ruby -}}</rt><rp>)</rp>
    {{- else -}}
      <rt></rt>
    {{- end -}}
  {{- end -}}
</ruby>
{{- end -}}
```

Usage:
`{{ ruby "日本語,にほんご;の;文法,ぶんぽう;は;難,むずか;しい"  }}`
