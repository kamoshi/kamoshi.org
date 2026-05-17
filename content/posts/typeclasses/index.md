---
title: Breaking apart the Haskell type class
date: 2023-11-02T17:28:25.466Z
icon: haskell
tags: [haskell, fp]
desc: >
  Type classes are perhaps the most distinctive feature of Haskell.
  I’ve found them pretty confusing, but in reality they are an incredibly
  elegant solution to a certain problem found in functional languages.
---

Type classes are perhaps the most distinctive feature of Haskell. I've found
them pretty confusing, but in reality they are an incredibly elegant solution to
a certain problem found in functional languages.

## Why?

In Haskell, we can write monomorphic functions which accept concrete types. As
an example, the function `add` shown below adds together two values of type
`Int`.

```haskell
add :: Int -> Int -> Int
add a b = a + b
```

We can also create functions which are polymorphic. The signatures of such
functions contain type variables. As an example, the function `head` shown below
has a type variable `a`, which means that it accepts lists of any type `a`.

```haskell
head :: [a] -> a
head []    = undefined
head (x:_) = x
```

However, there are situations where we want to create functions that accept a
specific set of different types. For example, a function should work with both
`Int` and `Float` values:

```haskell
a = 1 :: Int
b = 1 :: Float

x = add a a
y = add b b

add :: ? -> ? -> ?
```

Using a type variable like `a` in the function's signature would make it too
permissive, allowing unintended types like `String`.

```haskell
add :: a -> a -> a
add a b = a + b

-- wrong!
x = add "hello" "world"
```

To address this issue, Haskell introduces a concept called "type classes".

```haskell
a = 1 :: Int
b = 1 :: Float

add :: Num a => a -> a -> a
add a b = a + b
```

In this case the types `Int` and `Float` are instances of the `Num` type class.
By specifying the `Num a => ...` constraint in the function's signature, we
declare that the function should be polymorphic for instances of the `Num` type
class.

## How it works

You can visualize a type class as a set of operations defined for a given type.
The `Num` type class, for example, includes operations such as addition,
multiplication, and negation:

```haskell
class Num a where
  (+) :: a -> a -> a
  (*) :: a -> a -> a
  negate :: a -> a
  -- etc.

add :: Num a => a -> a -> a
add a b = a + b
```

In this class declaration, each operation is like a record field for the type.
It's as if there's a data type hidden behind the scenes, and each operation is a
selector function for that data type:

```haskell
data Num a = MkNum
  (a -> a -> a)
  (a -> a -> a)
  (a -> a)
  -- etc.

(+) :: Num a -> a -> a -> a
(+) (MkNum f _ _ ...) = f

add :: Num a -> a -> a -> a
add d a b = (+) d a b
```

Effectively, `Num a` serves as a table of operations for the `Num` class for
type `a`. This elegant mechanism allows Haskell to implement polymorphism with
type classes while maintaining strict type safety.

## Kinds

A type in Haskell is a classification that defines what kind of data a value can
represent. For example, `Int` is a type that represents integer values, and
`Float` is a type for floating-point numbers.

Kinds are a higher-level classification system for types in Haskell. Every type
in Haskell has a kind, and this kind represents the "type of types".

The notation for kinds in Haskell uses an asterisk `*` to represent the most
basic kind, which corresponds to concrete types. You can check the kind of any
type by using the `:kind` command in GHCi.

The kind of `Int` is `*`, indicating that `Int` is a concrete type.

```haskell
Int :: *
```

Similarly, the kind of `Float` is `*`, signifying that it is also a concrete type.

```haskell
Float :: *
```

However, Haskell's type system can go beyond basic types and deal with more
complex constructs. For instance, type classes introduce a kind called
`Constraint`, denoted as `=>`. The kind Constraint is used to represent
constraints on types and is commonly encountered when defining type classes and
their instances.

The kind of `Num` is `* -> Constraint`, showing that `Num` is a type class that
takes a type (like `Int`) as an argument.

```haskell
Num :: * -> Constraint
```

When `Num` is applied to `Int`, it becomes a constraint on the `Int` type,
indicating that `Int` is an instance of the `Num` type class.

```haskell
Num Int :: Constraint
```

Another example of a kind in Haskell is the kind of unary type constructors `*
-> *`. These unary type constructors are similar to generic types in other
languages because they accept a type parameter and produce a new type. The
signature `* -> *` signifies that the type constructor transforms one concrete
type into another concrete type.

An example of a trivial type constructor `Item` is shown below.

```haskell
data Item a = MkItem a

Item        :: * -> *
Item Int    :: *
Item String :: *
```

In this definition, `Item` is a unary type constructor because it takes a type
parameter `a` and produces a new type `Item a`, where `a` can be any concrete
type.

A commonly used example of a unary type constructor in Haskell is the `Maybe`
type. `Maybe` is used for representing optional values and handling potentially
missing or undefined data in a type-safe manner.

Its definition is as follows:

```haskell
data Maybe a = Nothing | Just a

Maybe :: * -> *
```

Haskell type constructors can be parameterized with more than one type variable,
as seen in the case of binary type constructors.

An example of a binary type constructor is the `Either` data type:

```haskell
data Either a b = Left a | Right b

Either            :: * -> * -> *
Either String     :: * -> *
Either String Int :: *
```

In this definition, `Either` is a binary type constructor because it requires
two type parameters, `a` and `b`, to create a new type, `Either a b`. The
`Either` type is often used for scenarios where a value can have one of two
possible types, such as representing success or failure or providing an
alternative to error handling. Conventionally, the `Left` value indicates error
conditions, while the `Right` value signifies valid or correct values.

### Higher Kinded Types

In Haskell, understanding the concept of "Higher Kinded Types" is crucial for grasping how certain more abstract type classes work.
HKTs involve type classes that are parameterized by type constructors, rather than just simple types.

The `Functor` type class is an example of a HKT class.
It defines two fundamental operations, `fmap` and `<$`, which operate on a type constructor `f`.

The Functor type class is defined as follows:

```haskell
class Functor f where
  fmap :: (a -> b) -> f a -> f b
  (<$) :: a -> f b -> f a

Functor :: (* -> *) -> Constraint
```

The `Functor` type class is not constrained to specific types but is rather parameterized by type constructors.
Its kind signature is `(* -> *) -> Constraint`, indicating that it works with type constructors.
This allows it to be applied to types like `Maybe`, `List`, or other container types that accept a type argument.

Similarly to `Functor`, the `Monad` type class is another example of a HKT type class.
It defines functions for monadic operations, such as `>>=`, `>>`, and `return`.

The definition of the `Monad` type class is as follows:

```haskell
class Monad m where
  (>>=)  :: m a -> (  a -> m b) -> m b
  (>>)   :: m a ->  m b         -> m b
  return ::   a                 -> m a

Monad :: (* -> *) -> Constraint
```

Like the `Functor` type class, the `Monad` type class is not tied to specific types but is parameterized by type constructors.
Its kind signature is also `(* -> *) -> Constraint`.

It is worth noting that a binary type constructor can often be partially
applied until it has the shape a type class expects. An example of this is the
`Functor` instance of `Either`:

```haskell
instance Functor (Either a) where
  fmap _ (Left x)   = Left x
  fmap f (Right x)  = Right (f x)
```

In the instance declaration above, we specify that, for any type `a`, `Either a` is an instance of the `Functor` type class.
This allows you to use the `fmap` function with `Either a`, providing a way to map over the second type argument of the `Either` type while keeping the first type argument fixed.

## What type classes buy us

At this point, the core idea is not that strange anymore.

A type class is a named collection of operations. A constraint such as `Num a`
means that the compiler must be able to find an implementation of that
collection for the type `a`. Once it has found one, functions can use the
operations from the class without knowing the exact concrete type.

This gives us a nice middle ground between two extremes:

- ordinary polymorphism, where a function must work for every possible type
- concrete functions, where a function works for exactly one type

Type classes let us say: this function works for any type, as long as the type
supports this small vocabulary of operations.

That is why they show up in so many parts of Haskell. `Eq` describes equality,
`Ord` describes ordering, `Show` describes conversion to text, `Num` describes
numeric operations, and `Functor` describes things that can be mapped over.
These are not inheritance hierarchies in the object-oriented sense. They are
more like named capabilities that the compiler can resolve statically.

## Instances and laws

There is one more important cultural detail. A type class declaration can say
which operations exist, but it usually cannot express all the rules those
operations should obey.

For example, `Eq` has this rough shape:

```haskell
class Eq a where
  (==) :: a -> a -> Bool
  (/=) :: a -> a -> Bool
```

The type checker can ensure that `(==)` returns a `Bool`, but it cannot prove
that equality is reflexive, symmetric, or transitive. It will happily accept a
bad instance:

```haskell
instance Eq Bool where
  True  == True  = False
  False == False = False
  _     == _     = True
```

This compiles, but it breaks the meaning people expect from `Eq`.

The same thing happens with `Functor`. The type of `fmap` says how values move
through a structure, but the usual functor laws are still a promise made by the
instance author:

```haskell
fmap id      == id
fmap (f . g) == fmap f . fmap g
```

This is part of the trade-off. Type classes are very good at making operations
available in a principled and type-safe way, but the deeper algebraic meaning of
those operations often lives in documentation, tests, and convention.

## Other languages

### Scala

Scala has a very direct encoding of the same idea. Instead of writing a Haskell
type class, we can write a trait parameterized by a type:

```scala
trait Show[A] {
  def show(value: A): String
}
```

An implementation of the type class is just a value of that trait:

```scala
given Show[Int] with {
  def show(value: Int): String =
    value.toString
}
```

Then a function can ask for the implementation with a `using` parameter:

```scala
def printValue[A](value: A)(using show: Show[A]): Unit =
  println(show.show(value))
```

This is very close to the dictionary-passing explanation from earlier. The
compiler finds a `Show[A]` value and passes it to the function. The main
difference is mostly syntactic and cultural: in Scala, the dictionary is a
normal value in the language, while in Haskell the type class machinery is more
specialized and deeply integrated into the compiler.

### Rust

Rust traits also cover part of the same territory:

```rust
trait Show {
    fn show(&self) -> String;
}

impl Show for i32 {
    fn show(&self) -> String {
        self.to_string()
    }
}

fn print_value<T: Show>(value: T) {
    println!("{}", value.show());
}
```

This gives Rust ad-hoc polymorphism: `print_value` works for any `T` with a
`Show` implementation.

Rust traits are not exactly Haskell type classes, though. Rust usually attaches
the type being implemented as the `Self` type of the trait, while Haskell type
classes are more naturally written as predicates over types:

```haskell
printValue :: Show a => a -> IO ()
```

Rust also has a strong coherence model: for a given trait and type, there should
be one clear implementation. This avoids many ambiguity problems, but it also
makes some patterns harder to express. Haskell has its own coherence story too,
but it has historically been more willing to grow extensions for type-level
programming, such as multi-parameter type classes, functional dependencies, type
families, and higher-kinded abstractions.

The practical takeaway is that Rust traits feel like a cousin of Haskell type
classes, not a direct copy. They solve a similar problem, but under different
constraints: Rust is also thinking about ownership, method syntax,
monomorphization, dynamic dispatch, and separate compilation.

## Final intuition

The simplest way to understand type classes is this:

> A type class is a promise that some operations exist for a type, and a
> constraint is a request for that promise.

Once you see that, the rest becomes less mysterious. `Num a => ...` means "give
me numeric operations for `a`". `Functor f => ...` means "give me mapping for
the type constructor `f`". `Monad m => ...` means "give me sequencing for `m`".

The machinery can become very sophisticated, especially once kinds,
higher-kinded types, and type-level programming enter the picture. But the core
idea stays surprisingly small: define a vocabulary of operations, implement it
for particular types, and let the compiler pass the right implementation to
polymorphic code.
