---
title: Building an SSG in Rust
date: 2024-10-05T12:29:54.111Z
desc: >
  Some reflections on using Rust to write a static site generator...
---

For the past few months in my spare time, I’ve been programming a simple
library in Rust that can be used to generate a static website from markdown
files and other assets. I myself use it to generate this very website, and it is
available under the GPL license on
[crates.io](https://crates.io/crates/hauchiwa), though it might be outdated -
the latest version is always on [GitHub](https://github.com/kamoshi/hauchiwa)
and the documentation is available on
[docs.rs](https://docs.rs/hauchiwa/latest/hauchiwa/).

You can add it to your own project in two ways, like so:

```rust
hauchiwa = "*"
hauchiwa = { git = "https://github.com/kamoshi/hauchiwa" }
```

## Background

Throughout the years I've tried many different tools, some of them better than
others.  For example, I really liked the speed of Hugo and the flexibility of
Astro, but none of the available tools fulfilled my needs. I wanted both speed
and lots of flexibility at the same time, I figured that I needed to create my own
generator from scratch to accomplish what I want to do.

Once I realized that I would be writing my own generator I had to choose the
language and the ecosystem for the generator, and as you already know, I ended
up with Rust. I've considered different languages and ecosystems, like Haskell,
but Rust currently has a lot of industrial momentum, that's the current
zeitgeist, so I started looking into writing a Rust implementation.

Contrary to what many people say these days, Rust is definitely not a silver
bullet, the fact that in Rust you have to deal with memory, even if it's
automatic most of the time, can be a deal breaker. Sometimes you just don't need
to care about memory, so having To deal with it is a waste of mental energy.
Nevertheless, I decided that this tradeoff is worth taking in this case, given
that Rust has:

- lots of good enough libraries
- vibrant community
- ergonomic abstractions
- automatic memory management with borrow checker

Ultimately I came to the conclusion that going with Rust will make it easy to
find any library I need to create a generator, and the memory management is an
acceptable tradeoff for the fact that Rust programs are generally quite fast and
compile to a single binary.

When it comes to the actual form factor of the library, I wanted it to be really
minimal and allow for maximal flexibility. I really enjoyed the way Astro works,
you use it as a general framework, you have lots of freedom to define each page
on the generated website. I would like to preserve this spirit in my library,
while at the same time creating a robust and idiomatic API in Rust.

Some of the requirements I had in mind are:

- The library should be decoupled from any templating engine, the user should
  be able to choose their own way to generate the HTML, they should even be able
  to do it by concatenating strings manually if they so desire.
- The user shouldn't be limited to Markdown, the library should be format
  agnostic and the user should be allowed to bring any parser they want and use
  it to convert any kind of file to HTML.
- The user should be able to generate HTML pages that don't have any original
  source files related to them, think of dynamically generated lists of pages,
  tags, etc.
- There should be a way to render different pages differently, some collections
  of Markdown files should output different looking pages.
- The library should make it easy to watch for changes and allow the user to add
  live reload while editing their website.

In my library I've tried to address all of these requirements, but it's still
being worked out. I've spent a lot of time thinking through lots of design
decisions until I landed on some sweet spot in the design space, but even now,
I'm not sure if there are better ways to accomplish some things...

## Thinking about the implementation

In my opinion, the hardest thing to come up with is the code architecture, how
to properly model the problem domain and not paint yourself into a corner. This
is especially true in Rust, because in this language there's quite a lot to
think about thanks to the borrow checker. I feel like using languages like OCaml
or F# would lend itself to iterating on the problem domain better, there you don't
have to think about references, pointers or ownership.

Don't get me wrong here; I think that Rust is an excellent programming language,
and I am happy to have it in my toolbox. However, some tools are better suited
to different tasks (think _law of the instrument_) and Rust seems like an
overkill when you shouldn't even have to care about the memory layout of a
struct.

I went with Rust though, so that made it considerably more difficult to iterate
on major changes to the structure of the program and how the various types
should compose. Each time I did an overhaul, I had to chase seemingly endless
waves of type checker and borrow checker errors. After each such overhaul the
program nearly always ran on the first try though - yay, strict type systems and
ADTs!.

The first issue I had to resolve was how to represent the data, all the
processed content files and assets. I first did it by creating a one-phase build
system where every single page on the website had to come from exactly just one
file in the filesystem. The upside of this approach was that it was easy to
trace the page back to the exact file it was generated from, and it was easy to
generate the entire website.

The downsides however were much worse:

- you couldn't generate page lists easily
- you couldn't generate pages not backed by any files easily
- live-reload rebuilds were messy
- generally ugly design

This was a major mistake on my part, the design I started with was clearly not
how websites work. The abstractions I came up with couldn't express the problem
domain without major hacks.

To fix this I decided to rewrite the library and implement a two-phase build
system, where the library first loads all content and assets and then runs
user-defined tasks that can create multiple concrete HTML pages. This rewrite
was quite of a headache, and took a lot of effort and time.

```rust
let website = Website::setup()
    .add_collections(vec![
       	Collection::glob_with::<Post>("content", "posts/**/*", ["md", "mdx"].into()),
    ])
    .add_styles(["styles".into()])
    .add_scripts(vec![
        ("search", "./js/search/dist/search.js"),
    ])
    // Task: generate posts
    .add_task(|sack| {
        sack.get_content_list::<Post>("posts/**/*")
            .into_iter()
            .map(generate_page)
            .collect()
    })
    .finish();
```

The new way to build pages turned out to be more flexible, so I think this
effort was well worth it in the end. The entire pipeline is much more simplified
now and allows for more granular incremental rebuilds with hashing and caching.

## Incremental build system

One of the more interesting things I ended up doing while implementing the new
build system was to create a custom incremental build process with live reload.
I've ended up reading an article called _Build systems à la carte_, which goes
over different ways to implent a build system in Haskell, I would recommend
reading it; it was really useful. Based on some prior experience as well as this
article, I've decided to go with a _suspending_ scheduler, as well as both
_verifying traces_ and _constructive traces_ for the rebuilding strategy.

_Suspending_ means that the moment a certain page requires, for example, a PNG
image or a CSS stylesheet I pause the page build process in order to prepare the
required asset. In practice, this just means I call a function that is supposed
to build that image, so it's not anything difficult.

```rust
  /// Get compiled CSS style by file path.
	pub fn get_styles(&self, path: &Utf8Path) -> Option<Utf8PathBuf> {
		let input = self.items.values().find(|item| item.file == path)?;
		if !matches!(input.data, Input::Stylesheet(..)) {
			return None;
		}

		self.tracked
			.borrow_mut()
			.insert(input.file.clone(), input.hash.clone());

		self.schedule(input)
	}
```

This function calls another function `schedule`, which builds the asset if it
needs to be built.

```rust
  fn schedule(&self, input: &InputItem) -> Option<Utf8PathBuf> {
		let res = self.builder.read().unwrap().check(input);
		if res.is_some() {
			return res;
		}

		let res = self.builder.write().unwrap().build(input);
		Some(res)
	}
```

Here `self.builder` is behind an `RwLock` which needs to be acquired in order to
build the asset. This is just an implementation detail; `RwLock` allows the
builder to be shared in a multithreaded environment and allows many reads at
the same time. This is optimal for the case when the asset is in fact already
built.

When it comes to the traces, I've decided to use the following strategy to trace
input assets:

```rust
#[derive(Debug)]
pub(crate) struct InputItem {
	pub(crate) hash: Vec<u8>,
	pub(crate) file: Utf8PathBuf,
	pub(crate) slug: Utf8PathBuf,
	pub(crate) data: Input,
}
```

Each asset has a binary `hash` and with this information alone we can easily
check if the input asset has changed in a meaningful way between two builds.

In order to trace the individual build tasks that are defined by the user to
generate the HTML pages, I've decided to use the following struct:

```rust
#[derive(Debug)]
struct Trace<G: Send + Sync> {
	task: Task<G>,
	init: bool,
	deps: HashMap<Utf8PathBuf, Vec<u8>>,
}
```

Here we have `task` which is in fact a closure pointer - a pointer to a function
defined by the user of the library. This function consumes a `Sack` which is the access point ftracks
the dependencies required by the task.

```rust
/// Task function pointer used to dynamically generate a website page.
type TaskFnPtr<G> = Arc<dyn Fn(Sack<G>) -> Vec<(Utf8PathBuf, String)> + Send + Sync>;

/// Wraps `TaskFnPtr` and implements `Debug` trait for function pointer.
pub(crate) struct Task<G: Send + Sync>(TaskFnPtr<G>);
```

These dependencies are then kept as the `deps` field, so we can check if any of
the input files required by a certain task have changed. If they have, we can
rebuild the task and update the dependencies. There's also the `init` field
which just forces the task to be built for the first time.

This is just the bare minimum to make this build system work, there are still
some open questions, like "What if the build task is nondeterministic, should it
be rebuilt every time?". Please take a look at the library code to see how the
current build system works in detail.

## Rusty type universe

While working on the library some features of Rust caught my eye, in particular
the fact that existential types and type erasure is so well integrated into the
language. I think this deserves its own section in here, because this is a quite
advanced concept that is rarely seen in most languages, In fact even in Haskell
you won't see this type of type magic used so prominently throughout the
ecosystem.

In Rust there are two flavors of existential types, one flaver is `impl` and the
other flavor is `dyn`. Generally speaking the difference is that when you write
`impl Trait` you are saying that you don't know what type will be here at
compile time, but you want the compiler to monomorphize this place, so at run
time it's just one type. When using `dyn Trait` you are saying that you want the
compiler to generate a table enabling dynamic dispatch at runtime, which has
runtime performance ramifications, but it also means that there is no
monomorphization.

```rust
fn function_impl_a(x: impl Trait) {
    // I can only use methods provided by Trait
}
```

Function using the `impl Trait` types will be generated multiple times in the assembly,
because each type will have to have its own copy of the function at runtime.

```rust
fn function_dyn_a(x: Box<dyn Trait>) {
    // I can only use methods provided by Trait
}
```

Functions using these `Box<dyn Trait>` types will be generated just once in the assembly,
but the methods will have to be dispatched at run time, which means more indirection.

In Rust there is a very interesting trait called `Any`. It seems useless at
first, because when you have a value of type `Box<dyn Any>` you seemingly can't do
anything with it...

There is just one thing you can do with a value like this, downcast it:

```rust
use std::any::Any;

fn main() {
    let items: Vec<Box<dyn Any>> = vec![
        Box::new(1),
        Box::new("abc"),
    ];

    for item in items {
        if let Some(num) = item.downcast_ref::<i32>() {
            println!("number: {}", num);
        } else if let Some(s) = item.downcast_ref::<&str>() {
            println!("string: {}", s);
        } else {
            println!("Unknown type");
        }
    }
}
```

I'll leave the potential usage of this powerful feature as an exercise for the
reader, if you can't come up with one - grep through my library :wink:

## Reflections

In Rust borrow checker makes it difficult to iterate on designs, so I think that
when first coming up with abstractions it's fine to use `.clone()` as much as
possible, this way you don't have to deal with memory when you aren't even sure
if the design will stick. Besides, using references is an optimization and
premature optimization is rarely a good idea, it's much better to only optimize
your program after you profile it and locate any bottlenecks.

I ended up learning a lot about the Rust type system, it's pretty powerful, but
it's also easy to come up with the wrong abstraction sometimes. Type classes, or
Rust traits, are very powerful, but they seldom make the right abstraction when
used liberally. I think most programs are better off avoiding new traits.

On the other hand, using traits to mark abstract capabilities is amazing and I
feel like more languages should have this feature in their type systems, in
particular the built-in traits like `Copy`, `Send` or `Sync`. I think that the
so-called lawful typeclass approach à la Haskell is fine, but the Rust
community really shows how useful type classes can be even without HKTs.

And to sum up, I feel like going with Rust was the pragmatic choice here, I
could have gone with some other language, and the solution might have been more
elegant or interesting, but the Rust solution works prefectly fine and I still
ended up learning a lot in the process. It's always worth noting that the choice
of a language means a lot more than just the language, you also end up choosing
the ecosystem of libraries, and the build tools used to build projects in that
language. I feel like Rust's world is a joy to work with.
