---
title: SSG that packs a punch
date: 2024-05-02T21:17:43Z
desc: >
    Let's create a yet another SSG from scratch, because that's just
    what this world needs!
---

Long story short, I decided to write my own static site generator in Rust. The
language choice is purely pragmatic and at first I considered using Haskell,
but alas the Rust ecosystem seems better suited for this task. Rust is not only
a very ergonomic and efficient language, but also has lots of high quality
packages to choose from.


## The why

I used many different SSGs made by other people, but over the years I realized
that they simply can't cut it for me. I used simple generators like Hugo, which
offer little to no support for JavaScript. Eventually I ended up using Astro,
which treats JavaScript as a first class citizen.

The thing I didn't like the most about Hugo, was that it used some strange,
clumsy templating language, which was quite annoying to use, but I enjoyed the
fact that nearly everything was built in, and the generation speed was fast.
When it comes to Astro, I enjoyed the fact that in templates you could write
TypeScript, as well the great flexibility with which you could arrange the
structure of the website. However, the JavaScript ecosystem is a mess, so using
that was not that pleasant overall.

Astro was closer to what I wanted, but not quite there.

With time, I realized that what I truly need is the flexibility of Astro,
coupled with a different language. Ideally a fun one like Haskell or Rust.

I started looking around and came across a couple of [blog
posts](https://arne.me/blog/write-your-own-ssg) detailing how people made the
move to their custom generator. This was the inspiration I needed.

I thought that even if it ends up quite bad in the end, at least I'll learn
*something*. And it would be a good chance to learn more Rust, so that's a win
right there. With Haskell it would be an ever bigger win, but hey, can't have
everything in life.


## The what

When trying to write a computer program it's a good idea to think about what it
should ideally be able to do. We should at least know the sketch of the
requirements before we start, so that we know if we are getting any closer to
the goal as we keep writing it.

When it comes to my generator I think I would like for it to have these
qualities:
- The generator must be *simple* and *extensible*
- HTML shouldn't be generated using some bespoke templating language, I should
have the full ecosystem at my disposal.
- Ideally I should not be limited to Markdown, I should be able to use
different file text formats as needed, e.g. Djot, AsciiDoc.
- Code snippets should be highlighted using Tree-sitter.
- Math should be pre-rendered via MathML without any client-side JavaScript.
- There should be a way to render different pages differently, some collections
of Markdown files should output different looking pages.
- I should be able to generate HTML pages that don't have any original source
files related to them, think dynamically generated lists of pages, tags.

With these requirements in mind I decided to go with Rust, because Rust has
lots of up-to-date parsers that can be used while building a generator. On top
of that I can use Tree-sitter without any need for complicated FFI or using
outdated libraries in other ecosystems.

When moving to a non-JavaScript backend for building SSG some issues related to
JavaScript inevitably start to crop up. There is some friction with using tools
like ESBuild, but I feel like the benefits far outweigh the challenges in the
end.


## The how
