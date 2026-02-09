---
title: Building a static site generator from scratch
date: 2026-02-02T20:28:32.821Z
desc: >
  This won’t be a guide how to build one, neither will it be about anything
  technical. In fact this is about the process itself which I had the chance to
  explore, with its ups and downs.
---

This won't be a guide how to build one, neither will it be about anything
technical. In fact this is about the process itself which I had the chance to
explore, with its ups and downs.

In 2021, I decided to build a website. Not just to have one, but to understand
how it all works - to have something to host, to learn the process from the
ground up, how each piece fits together, and how to make it work seamlessly.

That's because I always wondered how people build all these intriguing designs,
such as
- [gwern.net](https://gwern.net/)
- [ciechanow.ski](https://ciechanow.ski/)
- [craftinginterpreters.com](https://craftinginterpreters.com/)
- and so on...

## The beginning: raw HTML and CSS

I started the simplest way possible: a plain text editor. I hand-coded HTML and
wrote my own CSS files. It didn't take long to realize this approach was
painfully tedious. Every change required manual edits across multiple files. I
needed to automate the process of building a website.

## Hugo and Zola

Finding the right solution wasn't as straightforward as I had hoped. The
ecosystem is filled with static site generators, each taking a slightly
different approach to the same problem.

I first discovered **Hugo** in 2021. I learned it, built with it, but something
felt off. The tool was rigid - everything had to live in specific folders,
follow specific conventions. The templating language borrowed from Go, which I
had no interest in learning. It wasn't quite Jinja templates, just... different
and strange. I got my site working, but it never felt right. I kept searching.

Next, I found **Zola**, a Rust-based generator similar to Hugo but with some
improvements. It used Tera for templating, which was closer to Jinja. Better,
but still too opinionated, too much of a black box. Everything was
pre-configured, and extending it beyond its assumptions was difficult.

## The Astro experience

My search continued through 2021, 2022, and into 2023, when I discovered
**Astro**. I have to admit, I really liked Astro. The problem wasn't Astro
itself - it was the entire JavaScript/TypeScript ecosystem it belonged to.

Three major issues held me back:

1. **Performance**: JavaScript-based generators are inherently slower than they
   could be, especially when compared to Rust-based generators like Zola. They
   mostly have to rely on hot reloading, to keep the development experience
   fast.

2. **Dependency Quality**: The JavaScript ecosystem is... let's say it leaves
   much to be desired. Libraries in `node_modules` vary wildly in quality.
   TypeScript types are a coin flip, especially in niche packages. You can't
   rely on finding well-written, well-typed dependencies.

3. **Sustainability**: I don't trust JavaScript frameworks to exist in three,
   four, or five years. The churn is real - constant framework changes, breaking
   dependencies, new trends replacing old ones. Even though I'm writing content
   in Markdown, I had no confidence that running the generator in five years
   would work without wrestling with broken dependencies. Would Astro even
   exist?

## Building my own in Rust

In 2024, I decided to build my own generator in **Rust**. This turned out to be
far more challenging than I anticipated - not because of the language, I do like
Rust very much, but because designing the right architecture was genuinely
tricky. And as it turned out I haven't thought about what problem I was trying
to solve nearly enough.

### First attempt: two-phase model

My initial design was simple: two phases.

1. **Phase 1**: Load assets - a flat list of loaders for CSS files, images,
   Markdown articles, whatever.
2. **Phase 2**: Build HTML pages from the loaded assets.

Clean, simple, elegant. Or so I thought.

Then I needed to generate an XML sitemap from the built HTML pages. This
required a third phase, which I hadn't planned for. My architecture wasn't
flexible enough. The assumption was wrong.

### ...à la Carte

After nearly one year after coming up with the initial idea, I completely
rethought my approach. If some things depend on other things, and those things
depend on yet other things, everything should be a **graph**.

I discovered that build systems universally work this way. I found an excellent
paper called **"Build Systems à la Carte"** by researchers in the Haskell
community (written by Simon Peyton Jones among others). It described how to
design schedulers and build systems with specific properties - everything
represented as tasks with dependencies.

### The issue with granularity

I built a new system based on a dependency graph where any task could depend on
any other task. Asset loaders from the filesystem became first-class tasks like
everything else. It worked well, except for one problem: I lost granularity in
incremental rebuilds.

For example, if I had a list of all compiled CSS files, and one task needed just
a single file from that list, changing a *different* CSS file would trigger a
rebuild of that task anyway. The dependency wasn't fine-grained enough.

Something was still missing. I needed a better way to model the problem.

### Map-Reduce

I analyzed other static site generators - **Hakyll**, **Rib** and **Ema** in
Haskell, **Forester** in OCaml, and some others. I focused on the niche
generators, because I figured if I look at the popular ones I won't really find
anything that much insightful anyways.

What pushed me in the right direction was **Hakyll**, because I noticed how it
represents each item as a separate entity and it track these items each
separately with unique ID.

I didn't want to adopt this approach wholesale, but I realized that to properly
track changes for individual assets when loading from the filesystem, tasks
themselves needed to be granular. Each asset had to be tracked independently.

This led me to separate tasks into two types:

1. **Map tasks**: In other words, fine-grained tasks that for each input produce
   one output. A function applied to each file individually. We can track the
   hash of each file, trace how it flows through the task. When one file
   changes, we only rebuild what depends on that specific file.

2. **Reduce tasks**: Coarse-grained tasks that take many inputs, but produce one
   output. Aggregation or accumulation. These don't maintain one-to-one
   granularity but collect results from map tasks.

So it turns out that throughout all of this, after some experimentation and
iteration, I arrived at the **MapReduce model** seen maybe most prominently in
such things like Apache Hadoop and Spark where some tasks map and others reduce.

## Things I learned

This experience taught me several things:

- It's worth reading through academic papers when looking for inspiration and
  ideas. The "Build Systems à la Carte" paper crystallized concepts I'd been
  wondering about for some time.

- It's good idea to look at other existing solutions to the same or similar
  problems. Studying Hakyll, Ema, and others revealed ideas I wouldn't have
  discovered on my own.

- Usually the elegant abstractions are reflected in computer science and
  mathematics. MapReduce isn't just for static site generators. It's used in
  Apache Spark for big data processing. Good abstractions transcend their
  original domains.

- My initial two-phase design seemed perfect until it wasn't. Architectural
  decisions need iteration and refinement. Don't settle on first assumptions,
  sometimes things just need to be rewritten once or twice.

- The problems you're solving have likely been solved before, in different
  contexts. Find those solutions and learn from them. We all stand on the
  shoulders of giants.

---

This post represents several years of iterative design, failed assumptions, and
gradual improvements. In the end (is it though?), I feel it wasn't just about
building a tool. Honestly, it probably taught me more about how to think, and
how to learn, than about generating static sites and Rust.
