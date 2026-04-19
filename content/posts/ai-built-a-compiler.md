---
title: I used AI agents to build a compiler
date: 2026-04-19T11:15:28.386Z
tags: [ai, compiler]
desc: >
  I recently bought Claude Pro for a month just to see what the fuss was about.
  A week ago I decided to spend my leftover credits on building a small language
  compiler, and surprisingly it worked well enough.
---

I recently bought Claude Pro for a month just to see what the fuss was about. A
week ago I decided to spend my leftover credits on building a small language
compiler, and surprisingly it worked well enough.

## Building a language

I started with a few assumptions:
- I wanted record types with row polymorphism
- I wanted full inference, like Haskell or OCaml

Since then I've added a few more things:
- A trait system
- HKT

The vast majority of the type system code was implemented by models like Sonnet
4.6 and Opus 4.6, and the file that contains the type system is now around 3200
lines of Rust.

There's no way I would have been able to build something like this in a single
week without AI — it's just too much work. With AI I was able to get a PoC
language up quickly and iterate on it easily, even without fully understanding
what's going on in the code.

One could ask: if most of the code was generated, does that mean you don't
actually know anything about it? Did you learn anything at all?

I feel like I am learning something, even though AI is doing the heavy lifting.
I know the general layout of the project, I know what each part does *roughly*.
I know what the language can and can't do, its limitations and so on. By
designing the language and having AI implement it, I was able to quickly explore
the trade-offs of various designs, see what works and what doesn't. I was able
to architect the tooling around the language, the various pieces that connect
together to give you a nice experience writing code.

I now know, for example:
- How to create an LSP server
- How to create and use a tree-sitter grammar
- How to use LuaJIT as an embedded VM
- ...

I feel like even when using AI you can still learn *a lot* just by trying things
you wouldn't normally attempt on your own. If I had to build this language
completely by myself, I don't think I would ever reach a point where I'd have to
genuinely wonder about how row types and traits should interact while staying
sound, but here we are.

I think it's okay to be wary of AI, but I try to stay neutral. Like any
technology, it can be used for good or bad. And using AI to learn things you
otherwise never would have touched? That feels pretty squarely in the good
column.


## Where to see it

You can check the [online version][online] that just compile code to Lua, or you
can compile the [full version from the GitHub repo][github].

By the way, don't expect all things to work, this is just a language designed
and created in a single week!

[online]: https://kamoshi.org/projects/lume/
[github]: https://github.com/kamoshi/lume
