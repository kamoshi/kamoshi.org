# kamoshi.org

digital garden, blog, technical showcase, ...


## Architecture

### The engine

Custom static site generator built on [Hauchiwa](https://crates.io/crates/hauchiwa)

- Uses **Datalog** ([Crepe](https://crates.io/crates/crepe)) to compute
  backlinks, parent-child hierarchies, and co-citations at build time.
- Pure **Tree-sitter** implementation using raw grammars and custom SCM queries
  (`queries/`) for server-side highlighting without client-side overhead.
- Utilizes [Hypertext](https://crates.io/crates/hypertext) for
  compiled-to-Rust HTML templates.
- Markdown via `comrak` with manually added support for LaTeX
  (`pulldown-latex`), bibliographies (`hayagriva`), and custom shortcodes.
- Parallel processing via **Rayon** and intelligent caching.
- Client-side full-text search using **Pagefind**.
- Various client-side apps via prerendered Svelte components and random plain
  TypeScript scripts.


## Stack

- **Languages**: Rust, Deno, Wasm, TypeScript, Svelte 5, Datalog, Typst, etc.
- **Styling**: SCSS, Vanilla CSS.
- **Infrastructure**: Nix, Makefile, rsync.

## Development

```bash
# Setup environment
nix develop

# Build engine and site
make build

# Live reload development
make watch

# Performance profiling
make perf
```

## Structure

- `src/`: Core Rust engine (Datalog rules, Markdown processing, RSS).
- `content/`: Multi-lingual Markdown source files.
- `scripts/`: Svelte 5 components and TypeScript entry points.
- `queries/`: Tree-sitter highlight and injection queries.
- `tools/`: Rust CLI utilities.
- `styles/`: Global and component-specific SCSS.

## Deployment

Atomic deployments via `rsync` over SSH.

```bash
make deploy
```
