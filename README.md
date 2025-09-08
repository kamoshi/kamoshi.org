# Kamoshi.org - My digital home 🏡

Welcome! This is the source code for my personal website, kamoshi.org. Think of
it as my little corner of the internet, a place where I can experiment, share my
thoughts, and showcase some of the projects I've been working on.

I built this site with a very specific philosophy: to prioritize creative
freedom over polished professionalism. It's intentionally a bit rough around the
edges, a work in progress, because I believe a personal site should be a space
for genuine, unfiltered exploration.

## A glimpse into the site's content

The site is pretty eclectic, just like my interests! You'll find:
- A personal blog where I write about programming, languages, and whatever else is on my mind.
- A project showcase with detailed descriptions of my various creations.
- An interactive map that lets you explore my geotagged photos.
- Wiki-style notes on various topics, a kind of personal knowledge base.
- Slide presentations for various talks.
- Search functionality so you can easily find what you're looking for.
- And for those who prefer to follow along, a classic RSS feed for all the blog posts.

I made a deliberate choice to keep things minimal and avoid unnecessary code, so
you won't find a lot of bloat here. It's a very intentional approach to building
for the web.

## The tech under the hood 🛠️

Instead of a big, well-known platform, I decided to build the site with my own
custom static site generator, [Hauchiwa](https://docs.rs/hauchiwa), which I
wrote in Rust. This gave me complete control over the entire process, from
content processing to final output.

For the front end, I'm using a mix of modern tools like Svelte 5 and TypeScript
for interactive elements and SCSS for styling. The content itself is written
mostly in Markdown and processed with various libraries to handle everything
from code highlighting to academic bibliographies.

What's cool is that thanks to Hauchiwa's smart caching, the site only rebuilds
what's changed, making development incredibly fast. It's a polyglot site, too,
with content in English, Japanese, and Polish.

## Want to see how it's made?

The project is laid out pretty simply. The `content/` folder is where all the
writing lives, organized into different types like posts, projects, and wiki.
The `src/` folder contains the Rust code that powers the site generator, and the
`scripts/` and `styles/` folders hold the front-end components and stylesheets.

If you're curious about how it all works, you'll need Rust and the Deno runtime
installed. From there, it's just a few simple commands:
- `cargo run build` to build the site.
- `cargo run watch` to start a development server with live reloading.

You can also do:
- `make` to build the site.
- `make watch` to start a development server with live reloading.

Thanks to `flake.nix`, you can do:
- `nix develop` to enter a development environment with all dependencies installed.

The final, generated site ends up in the `dist/` directory.
