---
title: Ruby extensions for Markdown
date: 2023-01-15T16:19:00+01:00
tags: [Japanese, zola, hugo, astro]
draft: true
---
In Markdown there is currently no notation for adding ruby to East Asian scripts, however it is possible to add support for it by using shortcodes in SSGs.

## Examples
### Japanese
{日本語}(にほんご)の{文法}(ぶんぽう)は{難}(むずか)しい  
Remark: `{日本語}(にほんご)の{文法}(ぶんぽう)は{難}(むずか)しい`  
Tera: `{{/* ruby(expr="日本語,にほんご;の;文法,ぶんぽう;は;難,むずか;しい") */}}`  

### Chinese
{北}(Běi){京}(jīng)  
{北}(ㄅㄟˇ){京}(ㄐㄧㄥ)  
`{{/* ruby(expr="北,Běi;京,jīng") */}}`  
`{{/* ruby(expr="北,ㄅㄟˇ;京,ㄐㄧㄥ") */}}`

### Korean
{韓}(한){國}(국)  
`{{/* ruby(expr="韓,한;國,국") */}}`

### Vietnamese
{河}(Hà){內}(Nội)  
`{河}(Hà){內}(Nội)  `
`{{/* ruby(expr="河,Hà;內,Nội") */}}`

## Zola
The following is a snippet for the Tera templating engine which is inspired by Jinja2.
```jinja-html
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


## Astro
```ts
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
