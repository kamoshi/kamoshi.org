---
title: Hern, a small language experiment
date: 2026-05-17T15:00:00Z
tags: [compiler, plt, rust]
desc: >
  Hern is a small language I have been working on to see what kind of type
  system ideas I can put together in one coherent little world.
---

For the last few weeks I've been working on a small programming language called
Hern.

I don't really want to turn it into a real language, I'm not trying to create
the next Rust, or the next Haskell, or anything like that. The point is much
smaller and more personal: I wanted a playground where I could try some type
system ideas and see how they feel when they are all forced to live together in
one language.

It's one thing to read about [row polymorphism][row-polymorphism], traits,
higher-kinded types, [type inference][hindley-milner], or algebraic data types
in isolation. It's another thing to put them next to each other and ask: does
this still feel like one coherent language?

That is the interesting part to me.

Hern is less a new invention than a small collage of old ideas I like: ML-style
algebraic data types and inference, Haskell-style type classes, fixity, and
`do` notation, Rust-ish traits and tooling pragmatism, and row-polymorphic
records for structural data.


## The basic shape

At the surface, Hern is roughly in the ML/Rust family. It has sum types,
pattern matching, records, functions, traits, and inference.

A tiny Peano natural number type looks like this:

```hern
type Nat = Z | S(Nat)

fn zero() -> Nat {
  Z
}

fn succ(n: Nat) -> Nat {
  S(n)
}

fn add(lhs: Nat, rhs: Nat) -> Nat {
  match lhs {
    Z -> rhs,
    S(rest) -> succ(add(rest, rhs)),
  }
}
```

This is not exciting by itself, most functional languages can express this just
fine, but it is a good starting point because it immediately exercises a few
important things: recursive types, recursive functions, constructors, pattern
matching, and inference inside branches.

The goal is that the language should feel predictable here. `Nat` is a type, `Z`
and `S` are constructors, `add` is just a function, and recursive calls work
without any special ceremony.


## Records without ceremony

One of the ideas I wanted to try from the beginning was [row
polymorphism][row-polymorphism]. In practice, that means a function can ask for
a record with a particular field without caring about the other fields.

```hern
fn get_x(obj) {
  obj.x
}

let first = get_x(#{ x: "hello", y: 1 });
let second = get_x(#{ x: 42, z: true });
```

The function `get_x` does not need an interface, a class, or a named record
type. The type checker can infer that it accepts any record with an `x` field.


This is also where language design gets delicate, because records should compose
well with inference, but they should not make every error message impossible to
read. A lot of Hern has been about poking at that boundary.


## Traits as dictionaries

Hern also has traits. Internally, I think of them in the usual dictionary
passing way: a trait implementation is evidence that some operation exists for
some type. This is the classic [type class][type-class] story in a different
coat: constraints in the source language become dictionaries the compiler knows
how to pass around.

For example, equality for `Nat` can be written manually:

```hern
impl Eq for Nat {
  fn ==(lhs, rhs) {
    match lhs {
      Z -> match rhs { Z -> true, _ -> false },
      S(l) -> match rhs { S(r) -> l == r, _ -> false },
    }
  }

  fn !=(lhs, rhs) { !(lhs == rhs) }
}

let same = S(Z) == S(Z);
let different = S(Z) == Z;
```

What I like about this example is that it is recursive in two ways. The data
type is recursive, and the trait implementation is recursive as well. The
recursive call to `==` on the payload should simply find the implementation
currently being defined. That sounds obvious from the user's side, but it is
exactly the kind of thing that breaks if the internal model is not quite right.

This is why building a toy language is useful. You can very quickly discover
which parts of your mental model are vague.

I wrote about Haskell type classes separately in [Breaking apart the Haskell
type class][typeclasses-post], and Hern is partly me trying to make that model
feel concrete in a compiler I can hold in my head.


## Higher-kinded experiments

The more experimental part is higher-kinded traits. Hern's prelude has a small
`Functor` trait, which can be implemented for type constructors like `Option`,
arrays, or `Result(_, string)`.

```hern
fn show_int(x: int) -> string {
  to_string(x)
}

let maybe_text = Functor::map(Some(41), show_int);
let array_text = Functor::map([1, 2, 3], show_int);
let result_text = Functor(Result(_, string))::map(Ok(5), show_int);
```

Type constructors can be passed around at the type level, partially applied
types can participate in trait resolution, and the surface language can still
remain relatively small.

That is very much borrowing from the Haskell lineage: abstractions like
`Functor` become more interesting when they talk about type constructors rather
than concrete types, but the language still has to make the simple cases feel
simple.

That is the whole appeal of this project, I can just take ideas that usually
live in separate language communities and force them to interact.


## Small practical things

I also don't want Hern to be purely an exercise in type theory. If the language
is supposed to be pleasant to use, it needs a few boring constructs that make
ordinary programs easier to write.

For example, it has `let mut` and reassignment. I still like immutable values by
default, but sometimes a small loop with an accumulator is the clearest thing to
write.

```hern
fn sum_items(xs: [int]) -> int {
  let mut total = 0;

  for x in xs {
    total = total + x;
  }

  total
}

let total = sum_items([1, 2, 3, 4]);
```

The `for` loop also works with patterns, which makes it useful with tuples and
records.

```hern
let pairs = [(1, 2), (3, 4)];
let mut total = 0;

for (x, y) in pairs {
  total = total + x + y;
}

let rows = [#{ a: 5, b: 6 }];
let mut field_sum = 0;

for #{ a, .. } in rows {
  field_sum = field_sum + a;
}
```

There is also [`do` notation][do-notation]. This is one of those features that
looks a little magical until you remember that it is just syntax for chaining
monadic operations. I mainly added it because I wanted code that returns
`Option`, `Result`, or parser combinators to remain readable.

```hern
fn half_if_even(n: int) -> Option(int) {
  if n % 2 == 0 { Some(n / 2) } else { None }
}

let result = do {
  let a <- half_if_even(8);
  let b <- half_if_even(a);
  Some(a + b)
};
```

A thing I came to realise is that language can have a very clever type system,
but if the everyday code is unpleasant to write, the cleverness does not help
very much.


## Operators and dependencies

Hern lets functions and trait methods define arbitrary operators with explicit
[fixity and precedence][haskell-fixity]. This is mostly inspired by Haskell,
but I wanted it to fit together with the trait system rather than be a separate
parser trick.

```hern
fn infixl 9 <+>(a: int, b: int) -> int {
  a + b
}

let result = 1 <+> 2 <+> 3;
```

The prelude uses the same mechanism for normal arithmetic operators. Addition is
not hard-coded as one special operation on one type. It is a multi-parameter
trait with a [functional dependency][functional-dependencies] from the input
types to the output type.

```hern
trait Add 'lhs 'rhs -> 'output {
  fn infixl 6 +(lhs: 'lhs, rhs: 'rhs) -> 'output
}

fn add_twice(x: 'a, y: 'a) -> 'a
where 'a 'a -> 'a: Add
{
  x + y + y
}
```

The `-> 'output` part is the functional dependency. It says that once the left
and right argument types are known, the output type is determined. Without that
kind of rule, an expression like `x + y` can become ambiguous very quickly. The
Haskell world has a long-running parallel design conversation around
[functional dependencies and type families][fundeps-vs-type-families]; Hern is
only using the small piece it needs here.

The same shape is useful for indexing:

```hern
trait Index 'receiver 'key -> 'output {
  fn get(receiver: 'receiver, key: 'key) -> 'output
}
```

An array indexed by an `int` produces an element, a map indexed by a key
produces a value, and a string indexed by a range produces a string. These are
all the same operation from the user's point of view, but the type system still
has enough information to know the result type.

This is one of the places where Hern feels closest to what I wanted from the
project. Operators, traits, and inference are not separate features bolted next
to each other, they all describe the same underlying idea: some operation is
available for some types, and resolving that operation should determine the rest
of the expression.


## The compiler

The compiler is written in Rust and currently targets [LuaJIT][luajit], and
that choice is mostly pragmatic. Lua is small, embeddable, fast enough for this
experiment, and easy to generate, LuaJIT is a simple target that has lots of
useful features.

The tooling around the language is already more complete than I expected:

- a compiler
- a standard prelude
- a test runner
- an LSP server
- a [tree-sitter grammar][tree-sitter-hern]
- editor integrations
- documentation hovers on this website

This website now analyzes Hern snippets while rendering Markdown, so the code
blocks are highlighted with tree-sitter, but the hover types come from the Hern
compiler itself. It means the compiler is not only a binary you run in a
terminal, but also a library that can feed other tools.

One thing that feels especially nice is that most of the tooling still lives in
a single small binary, including the embedded LuaJIT runtime. That keeps the
project easy to move around: the compiler, test runner, language tooling, and
runtime all travel together.

[do-notation]: https://en.wikibooks.org/wiki/Haskell/do_notation
[functional-dependencies]: https://wiki.haskell.org/Functional_dependencies
[fundeps-vs-type-families]: https://wiki.haskell.org/Functional_dependencies_vs._type_families
[haskell-fixity]: https://www.haskell.org/tutorial/functions.html
[hindley-milner]: https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system
[luajit]: https://luajit.org/luajit.html
[row-polymorphism]: https://en.wikipedia.org/wiki/Row_polymorphism
[tree-sitter-hern]: https://crates.io/crates/tree-sitter-hern
[type-class]: https://en.wikipedia.org/wiki/Type_class
[typeclasses-post]: /posts/typeclasses/
