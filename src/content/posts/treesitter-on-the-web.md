---
title: Bringing treesitter to the Internet
date: 2024-02-14T18:32:41.645Z
desc: >
    Are we stuck using regex based syntax highlighters forever?
    What if there is an alternative we could use on the web?
    In this article I try to use a real parser to highlight syntax in code blocks.
---

Recently, there has been a complete rewrite of [Shiki](https://github.com/shikijs/shiki),
a rather nice syntax highlighter that you can employ to accentuate a plethora of code on your online blog.
It's built upon the very same system utilized within VS Code - [TextMate grammars](https://github.com/microsoft/vscode-textmate).
Additionally, a side effect of this setup is its reliance on the [Oniguruma](https://en.wikipedia.org/wiki/Oniguruma) regex library,
which is not so nice for a number of reasons. It works by using glorified regexes to highlight the syntax.

A crazy thought struck me at some point. What if I used [Treesitter](https://tree-sitter.github.io/tree-sitter/) to do all the
heavy-lifting of finding out how to color the syntax of the code snippets?
Treesitter is a novel approach to syntax highlighting which is utilized in editors
like Neovim, Helix or Zed. Being a Neovim user myself, I was quite enthusiastic
about the thought of being able to use the same tool used by Neovim.

This website is generated statically by Astro, which in turn runs on Node.js,
at least that's the setup for the time being. If I wanted to use Treesitter
I would have to somehow plug Treesitter into the Node.js process...

My language of choice for doing this is Rust, because it's a systems language,
it has some of the best bindings to Treesitter, and it has pretty good interop
with the JavaScript ecosystem.


## WASM is too hard

My first intuition was to try to compile Treesitter into a WASM module,
however this proved to be much harder than I anticipated at first.

The main problem is compiling `tree-sitter` crate to `wasm32-unknown-unknown`.
This is simply impossible to do without resorting to hacks, because there's
a C header in the source code which cannot be compiled while passing that target.

Another problem is that there is currently an [ABI mismatch between C and Rust](https://github.com/rust-lang/rust/issues/71871)
when it comes to the `wasm32-unknown-unknown` target. The `wasm32-wasi` target
is not affected by this issue.


## The native approach

I've found that there are pretty good [Rust bindings to the Native API for Node](https://napi.rs/),
so I decided to try loading Treesitter compiled as a dynamic library.

A lot of the work related to the setup, as well as the building can be automated
using a [CLI tool](https://napi.rs/docs/introduction/getting-started),
which I definitely recommend checking out.

For this type of project we have to set the crate type as a dynamic library respecting the C ABI.

```toml
[lib]
crate-type = ["cdylib"]
```

Next we need to import all the external libraries required to compile for Node.

```toml
[dependencies]
# Default enable napi4 feature, see https://nodejs.org/api/n-api.html#node-api-version-matrix
napi = { version = "2.12.2", default-features = false, features = ["napi4"] }
napi-derive = "2.12.2"
```

In this section we define the dependencies required for the project. We need to 
use the `napi` crate. The feature flag indicates compatibility with Node.js N-API version 4.
The comment provides a link to the Node.js documentation explaining the N-API version matrix.

```toml
[build-dependencies]
napi-build = "2.0.1"
```

In this section, we define build dependencies. Build dependencies are dependencies
that are only needed during the build process, such as compiler plugins or code generation tools.

```toml
[profile.release]
lto = true
strip = "symbols"
```

We can add Treesitter dependencies like this.

```toml
[dependencies]
# Treesitter
tree-sitter = "0.20.10"
tree-sitter-highlight = "0.20.1"

# Languages
tree-sitter-astro = { git = "https://github.com/virchau13/tree-sitter-astro.git", rev = "e924787e12e8a03194f36a113290ac11d6dc10f3" }
tree-sitter-css = "0.20.0"
tree-sitter-html = "0.20.0"
tree-sitter-javascript = "0.20.3"
```

In Rust we need to create a function which will be callable from the JavaScript
side, it needs to be marked by the `#[napi]` procedural macro. It makes the function
visible to Node.js. By the way, there are only some types you can use in such a function.
Check the documentation for the available types.

```rust
#[macro_use]
extern crate napi_derive;
```

In the entry function I load the configuration for a language, if there's no
such language then we need to return early. Next, we create a highlighter and pass
the config, a source code we want to highlight, as well as a callback used for
retrieving additional language configs. This is needed for handling injections.
Last we convert events into a type which can be converted into a JavaScript object.

```rust
#[napi]
pub fn hl(lang: String, src: String) -> Vec<HashMap<String, String>> {
    let config = match configs::get_config(&lang) {
        Some(c) => c,
        None => return vec![
            HashMap::from([
                ("kind".into(), "text".into()),
                ("text".into(), src.into())
            ])
        ]
    };

    let mut hl = Highlighter::new();
    let highlights = hl.highlight(
        &config,
        src.as_bytes(),
        None,
        |name| configs::get_config(name)
    ).unwrap();

    let mut out = vec![];
    for event in highlights {
        let event = event.unwrap();
        let obj = map_event(event, &src);
        out.push(obj);
    }
    out
}
```

The events we get from Treesitter need to be converted into something serializable, e.g. `HashMap`.

```rust
fn map_event(event: HighlightEvent, src: &str) -> HashMap<String, String> {
    match event {
        HighlightEvent::Source {start, end} => HashMap::from([
            ("kind".into(), "text".into()),
            ("text".into(), src[start..end].into())
        ]),
        HighlightEvent::HighlightStart(s) => HashMap::from([
            ("kind".into(), "open".into()),
            ("name".into(), captures::NAMES[s.0].into())
        ]),
        HighlightEvent::HighlightEnd => HashMap::from([
            ("kind".into(), "close".into())
        ]),
    }
}
```

On the JavaScript side we need to load the Rust library like this, assuming we
are writing ES modules that is. This `require` is *required* (heh) to be able to import the
library, but we have to create it ourselves.

```javascript
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
export const { hl } = require('./treesitter.linux-x64-gnu.node');
```

Once we have this library we can load it inside Node (almost) like any other module.

```node
Welcome to Node.js v21.6.1.
Type ".help" for more information.
> const treesitter = await import('./dist/index.js')
undefined
> treesitter.hl.toString()
'function hl() { [native code] }'
> treesitter.hl('ts', 'function a() {}')
[
  { name: 'keyword', kind: 'open' },
  { text: 'function', kind: 'text' },
  { kind: 'close' },
  { kind: 'text', text: ' ' },
  { kind: 'open', name: 'function' },
  // ...
]
```

As you can see by the `[native code]` marker when calling `toString()` on the `hl`
function, this function is written using a compiled language.

We can use this function for example inside a remark plugin, to transform the
tree of elements we get from parsing a markdown file.

Below is an example which I used to highlight syntax in all code blocks.

```typescript
export default function rehypeTreesitter() {
  return function (tree: any) {
    visit(tree, null, (node, _, above) => {
      if (node.tagName !== 'code' || above.tagName !== 'pre') return;
      const code = node.children?.[0].value || '';
      const lang = node.properties.className?.[0].replace('language-', '') || '';
      const parent = { ...above };

      above.tagName = 'figure';
      above.children = [parent];
      above.properties = {
        className: 'listing kanagawa',
        ...!!lang && { "data-lang": lang },
      };

      const root = { children: [] };
      const ptrs: any[] = [root];

      for (const event of treesitter.hl(lang, code)) {
        switch (event.kind) {
          case 'text': {
            const inserted = text(event.text);
            ptrs.at(-1).children.push(inserted);
          } break;
          case 'open': {
            const inserted = span(event.name);
            ptrs.at(-1).children.push(inserted);
            ptrs.push(inserted);
          } break;
          case 'close': {
            ptrs.pop();
          } break;
        }
      }

      node.children = root.children;
    });
  };
}
```


## Extensions

Using Treesitter means that we can easily add new language parsers, and write custom
queries for highlights and injections.

For example if we want to have syntax highlighting for Astro, we have to install
parsers, which need to be included in the `Cargo.toml` file. Then we can set up them
like this.

```rust
pub static CONFIGS: Lazy<HashMap<&'static str, HighlightConfiguration>> = Lazy::new(|| {
    HashMap::from([
        (
            "astro",
            config_for(
                tree_sitter_astro::language(),
                query!("astro/highlights"),
                query!("astro/injections"),
                ""
            )
        ),
        (
            "html",
            config_for(
                tree_sitter_html::language(),
                tree_sitter_html::HIGHLIGHTS_QUERY,
                tree_sitter_html::INJECTIONS_QUERY,
                "",
            )
        ),
        // -- snip --
    ])
})
```

In the previous snippet I've used some custom-made things, as well as used `once_cell`
just to statically load all the configurations right at the beginning.
I've used here a function `config_for` which is a simple wrapper around `HighlightConfiguration::new(...)`
and a `query!` macro.

The macro just loads a string from a file and embeds it as a static string.

```rust
macro_rules! query {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/queries/",
            $path,
            ".scm"
        ))
    };
}
```

This is useful for when you would like to write a custom query, because the one
included with the parser is not good enough, or if there is none at all.

Below is an example `highlights.scm` query for Astro, which adds syntax highlighting
captures for all `.astro` files. I've taken them from `nvim-treesitter` repo.
Be careful, however, not every directive available in Neovim is available
for general use. For example, `#lua-match?` is Neovim only and won't work here.

```query
(tag_name) @tag
(erroneous_end_tag_name) @keyword
(doctype) @constant
(attribute_name) @property
(attribute_value) @string
(comment) @comment

[
  (attribute_value)
  (quoted_attribute_value)
] @string

"=" @operator

[
  "{"
  "}"
] @punctuation.bracket

[
  "<"
  ">"
  "</"
  "/>"
] @tag.delimiter
```

As for the injections, we can add a `injections.scm` file. This will allow us
to highlight additional languages embedded inside Astro, like TypeScript, or HTML.

```query
(frontmatter
  (raw_text) @injection.content
  (#set! "injection.language" "typescript"))

(interpolation
  (raw_text) @injection.content
  (#set! "injection.language" "tsx"))

(script_element
  (raw_text) @injection.content
  (#set! "injection.language" "typescript"))

(style_element
  (raw_text) @injection.content
  (#set! "injection.language" "css"))
```

Last but not least, you have to configure the styles for the classes generated
by Treesitter. Some would argue this is in fact the hardest part, which is why
I've borrowed the color scheme from the [Kanagawa theme](https://github.com/rebelot/kanagawa.nvim) for Neovim :)

```scss
// Identifiers
.variable-builtin { color: var(--kngw-waveRed); }
.variable-parameter { color: var(--kngw-springViolet2); }

.constant { color: var(--kngw-surimiOrange); }
.constant-builtin { color: var(--kngw-surimiOrange); }

.label { color: var(--kngw-oniViolet); }

// Literals
.string { color: var(--kngw-springGreen); }
.string-special { color: var(--kngw-boatYellow2); }

.number { color: var(--kngw-sakuraPink); }
.number-float { color: var(--kngw-sakuraPink); };
```

## The result

If everything worked correctly you should be able to see a nicely highlighted
snippet below :)

```astro
---
const { isRed } = Astro.props;
---
<!-- If `isRed` is truthy, class will be "box red". -->
<!-- If `isRed` is falsy, class will be "box". -->
<div class:list={['box', { red: isRed }]}><slot /></div>

<style>
  .box { border: 1px solid blue; }
  .red { border-color: red; }
</style>
```

All the code, which I do use in production and might change in the future, is 
[available on Github](https://github.com/kamoshi/kamoshi.org/tree/main/tools/treesitter).
