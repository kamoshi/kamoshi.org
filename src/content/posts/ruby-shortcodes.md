---
title: Japanese Ruby shortcodes for Zola & Hugo
date: 2023-01-15T16:19:00+01:00
tags: [Japanese, zola, hugo]
draft: true
---

In Markdown there is currently no notation for adding ruby to East Asian scripts, however it is possible to add support for it by using shortcodes in SSGs.

## Examples
### Japanese
{{ ruby(expr="日本語,にほんご;の;文法,ぶんぽう;は;難,むずか;しい") }}  
`{{/* ruby(expr="日本語,にほんご;の;文法,ぶんぽう;は;難,むずか;しい") */}}`  

### Chinese
{{ ruby(expr="北,Běi;京,jīng") }}  
{{ ruby(expr="北,ㄅㄟˇ;京,ㄐㄧㄥ") }}  
`{{/* ruby(expr="北,Běi;京,jīng") */}}`  
`{{/* ruby(expr="北,ㄅㄟˇ;京,ㄐㄧㄥ") */}}`

### Korean
{{ ruby(expr="韓,한;國,국") }}  
`{{/* ruby(expr="韓,한;國,국") */}}`

### Vietnamese
{{ ruby(expr="河,Hà;內,Nội") }}  
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
