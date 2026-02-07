---
title: Why I use Rust
date: 2026-02-07T20:20:58.714Z
---

At work, I use the usual suspects, languages like TypeScript, Python, C#. The
boring, table-stakes languages that everyone expects you to know. But for my
hobby projects, when I'm building something just for fun, at this point I reach
for Rust nearly every single time.

Why Rust? Can I even justify this choice? I think I can.


## What makes Rust different

In my opinion, Rust is one of the best choices you can make today when picking a
language. It's modern, with fantastic syntactic constructs and a wonderful type
system. But here's the key thing for me: **it has sum types**.

Sum types are my litmus test for whether a language is modern. If a language
doesn't have sum types, that's a serious deficiency.

TypeScript can express a lot in its type system, but because it's fundamentally
a wrapper around JavaScript, it's still riddled with potential bugs. The worst
codebases are those where teams slap `any` everywhere. It becomes an absolute
mess and it's incredible how bad it can get sometimes.

In Rust, everything is properly typed. Sure, you can use `unwrap` or `expect`
and get a panic, but that's not really a big problem in practice. What makes
Rust special is its range of applications: you can write high-level code at the
same abstraction level as garbage-collected languages, but you can also drop
down to low-level code with lifetimes and memory control. References to
variables have lifetimes, and the compiler enforces these rules. You can't just
return a reference from a function that points to an object on the stack. When
you need even more control, unsafe blocks let you get close to C.


## The universal language

What's amazing is that you can use Rust for essentially anything. Operating
system kernels, device drivers, web servers, command-line tools, GUI
applications, video games. The language scales to all of these domains. Once you
know Rust well, you can tackle virtually any type of project without feeling
limited by the language itself.


The abstraction level is also well-balanced. Unlike C, where you manage every
detail manually, Rust's compiler handles memory allocation and deallocation
automatically. The code feels high-level while maintaining low-level
performance.


## The ecosystem actually works

What I really appreciate about Rust is that when you need a library in Rust,
there's a good chance it exists on crates.io and you can add it to your project
with Cargo. The build system is straightforward: `cargo new` to start a project,
`cargo build` to compile, `cargo test` to run tests. No wrestling with build
configuration or dependency management. The whole Rust toolchain is excellent.

This is a double-edged sword. Like `npm` in the JavaScript world, Rust projects
can accumulate hundreds of transitive dependencies. A project might pull in 600
crates total, which creates maintenance concerns, because someone has to keep
those libraries updated and fix vulnerabilities. But the practical reality is
that it works smoothly enough that the dependency tree rarely becomes a problem
you need to think about.

Python has similarly easy dependency management, but lacks static typing by
default. The recent addition of type hints helps, but you can't be sure if the
libraries you're using are well-typed. You often end up debugging with print
statements to understand how a library actually behaves. Rust's type system
eliminates this entire class of problems, and the built-in documentation tools
mean libraries tend to have better docs too.


## Why not other languages?


Before settling on Rust, I looked at what else was available. I wanted a
language with good library support that I could use for any kind of project,
from quick scripts to larger programs and libraries.

Python? Sure, I know there will always be a library for everything, but Python
doesn't have proper static typing. And with those libraries I'm pulling in, I
don't know if they'll be properly typed or not. Also the performance leaves a
lot to be desired.

TypeScript has improved significantly along with the `npm` ecosystem, but you
still can't be certain about type quality or long-term maintenance. The
JavaScript world churns through frameworks and tools at an exhausting pace -
Astro, new versions of Svelte, constant API changes. If you build something in
TypeScript today, you'll spend the next five years updating dependencies and
adapting to breaking changes. Rust feels far more stable by comparison.

OCaml and Haskell are theoretically compelling. They have strong type systems
and functional programming paradigms. But both suffer from small communities and
sparse library ecosystems. For common tasks like building an API server, Haskell
has solid options like Servant. But venture into niche territory and you'll find
abandoned libraries, unmaintained for years. Your options become: write it
yourself from scratch, or fork and update an old project. That's kind of sad,
but that's how it is.

Rust doesn't have this problem. The community is active enough that gaps get
filled. I've worked on some fairly niche projects and consistently found what I
needed: `grass` for compiling SCSS to CSS, `image-rs` for image manipulation and
format conversion. The libraries are well-maintained, actively developed, and
actually documented.


Part of this might be the fact that Rust is still relatively young, stable for
about a decade, which means most libraries were written recently and haven't had
time to be abandoned. But I don't think that will change. Rust has serious
institutional backing from Microsoft, Google, and others who are investing
heavily in rewriting system components and kernels. The language has proven too
useful and has had too much invested in it to fade away. If you learn Rust
today, it will likely remain relevant for decades.


## The testing story

Rust has unit testing built into the language in a genuinely useful way. You can
write a private function in a module, then create a `mod tests` submodule right
there in the same file to test it. Run `cargo test` and it works, no need to
make functions public just for testing, no need to set up separate test
infrastructure.

This might seem minor, but it's a quality-of-life improvement that should be
standard. In most languages like JavaScript with Mocha, Java with JUnit, Haskell
with HUnit, and so on, you have to export functions to test them, polluting your
public API. Rust lets you keep implementation details private while still
testing them thoroughly.


## The C++ problem

C++ deserves special mention as a cautionary tale. C itself makes sense to me.
It's nearly 60 years old, designed for one thing, and still does it well at the
lowest levels of system programming. But C++ tried to be everything: classes,
objects, templates, and now increasingly desperate attempts to bolt on modern
safety features like Rust-style traits (concepts). The language has accumulated
so much complexity that learning it thoroughly requires years of study.

If you don't already need to work in C++, I'd argue you shouldn't learn it. The
better path forward is to learn C for low-level work, Rust for safety and
expressiveness, and maybe Zig for a modern C alternative. That trio covers the
same ground as C++ without the accumulated baggage.


## The OOP hangover

Languages like C#, Java, TypeScript, and JavaScript emerged from an
industry-wide bet on object-oriented programming. For a time, everyone became
convinced that OOP was the solution to software complexity. The entire industry
aligned around this paradigm.

Years later, it's clear that OOP wasn't the best path toward maintainable, safe
software. The goal was always to write good software - language features are
means to that end, not ends in themselves. But we needed better tools for
expressing program structure, and OOP's class hierarchies turned out to be less
helpful than features like sum types for modeling domains accurately.

Now the industry is stepping back. Rust has traits and structs instead of
inheritance hierarchies. Swift has protocols that work similarly to Rust's
traits, though the language has accumulated enough features that it risks
becoming unwieldy like C++. Kotlin improved on Java by adding functional
features. Python and TypeScript both retrofitted type systems onto dynamic
languages. The pattern is clear: types matter, static analysis matters, and
expressive type systems help build software that lasts longer than a decade.

The problem is that we're still living with the consequences of that OOP bet. If
you want to choose a language today, you're stuck between established giants
that carry decades of design baggage, or newer languages with uncertain futures.
The exceptions - Swift backed by Apple, Kotlin backed by Google and JetBrains,
Rust backed by Mozilla Foundation and now major tech companies - have
institutional support that makes them safer bets.


## Why Rust, then

Rust is modern, practical, and well-supported. It has the expressiveness I need
without fighting the language's design. The toolchain is excellent, the
community is active, libraries exist for most use cases, and I can apply it to
everything from embedded systems to web servers to GUI applications.

Most importantly, learning Rust feels like a good investment. The language isn't
going anywhere, and once you know it well, you have a tool that works for nearly
any programming task. That's worth something.
