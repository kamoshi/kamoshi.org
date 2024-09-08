---
title: Lambda calculus is the DNA of all computation
date: 2024-09-07T20:32:00.281Z
desc: >
  Lambda calculus can be used to express any computation, but what does it entail? As it turns out first class functions are the single most powerful abstraction.
scripts:
  - lambda
---

In lambda calculus the entire grammar of the language comprises of just three kinds of expressions

1. variables
2. lambda abstraction
3. application

The grammar can be formulated recursively as

```
t ::=
  x     // variable
  λx.t  // lambda abstraction
  t t   // application
```

In lambda calculus functions accept only a single argument, but they can be nested, this is referred to as currying in the literature. For example, we can have a simple function that just returns its argument $\lambda x. x$ which is generally known as the identity function.

Any language that has first class functions with closures can be used to simulate lambda calculus. As an example I will use my own custom toy language.

```
// variable declaration
let a = 10;

// function declaration (id)
let f x = x;

// anonymous function (id)
let g = fn x -> x;

// function application
f g 1
```

The result of evaluating `f g 1` is `1`, because `f g` → `g` and `g 1` → `1`.

As we can see the language can be used to express every single term we have in lambda calculus. Can these terms be used to express any computation? As it turns out yes, in fact we can encode data type using just functions. These encodings are called [Church encodings](https://en.wikipedia.org/wiki/Church_encoding) in the literature.

Let's start with booleans, they can be defined as follows:

$$
\begin{align}
  \text{tru} = \lambda \text{t}.\: \lambda \text{f}.\: \text{t}; \\
  \text{fls} = \lambda \text{t}.\: \lambda \text{f}.\: \text{f};
\end{align}
$$

```
let tru t f = t;
let fls t f = f;
```

And then we can defined a function that will work just like `if ... then ... else ...` in general purpose programming languages.

$$\text{test} = \lambda \text{l}.\: \lambda \text{m}.\: \lambda \text{n}.\: \text{l}\, \text{m}\, \text{n};$$

```
let test l m n = l m n;
```

Let's also defined the `and` combinator which checks if two values are true.

$$\text{and} = \lambda \text{b}.\: \lambda \text{c}.\: \text{b}\, \text{c}\, \text{fls};$$

```
let and_ b c = b c fls;
```

Let's see if this works! Feel free to play around with the code...

<pre class="flox-eval">
let tru t f = t;
let fls t f = f;

let test l m n = l m n;
let and_ b c = b c fls;

test (and_ tru tru) "both true!" "nope"
</pre>
