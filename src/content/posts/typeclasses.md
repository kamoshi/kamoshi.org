---
title:  Breaking apart the Haskell type class
date:   2023-11-02T17:28:25.466Z
icon:   haskell
desc: >
    Type classes are perhaps the most distinctive feature of Haskell.
    I’ve found them pretty confusing, but in reality they are an incredibly
    elegant solution to a certain problem found in functional languages.
---

Type classes are perhaps the most distinctive feature of Haskell.
I've found them pretty confusing, but in reality they are an incredibly elegant solution to a certain problem found in functional languages.

## Why?

In Haskell, we can write monomorphic functions which accept concrete types.
As an example, the function `add` shown below adds together two values of type `Int`.

```haskell
add :: Int -> Int -> Int
add a b = a + b
```

We can also create functions which are polymorphic.
The signatures of such functions contain type variables.
As an example, the function `head` shown below has a type variable `a`, which means that it accepts lists of any type `a`.

```haskell
head :: [a] -> a
head []    = undefined
head (x:_) = x
```

However, there are situations where we want to create functions that accept a specific set of different types.
For example, a function should work with both `Int` and `Float` values:

```haskell
a = 1 :: Int
b = 1 :: Float

x = add a a
y = add b b

add :: ? -> ? -> ?
```

Using a type variable like `a` in the function's signature would make it too permissive, allowing unintended types like `String`.

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
By specifying the `Num a => ...` constraint in the function's signature, we declare that the function should be polymorphic for instances of the `Num` type class.

## How it works

You can visualize a type class as a set of operations defined for a given type.
The `Num` type class, for example, includes operations such as addition, multiplication, and negation:

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
It's as if there's a data type hidden behind the scenes, and each operation is a selector function for that data type:

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

Effectively, `Num a` serves as a table of operations for the `Num` class for type `a`.
This elegant mechanism allows Haskell to implement polymorphism with type classes while maintaining strict type safety.


## Kinds

A type in Haskell is a classification that defines what kind of data a value can represent.
For example, `Int` is a type that represents integer values, and `Float` is a type for floating-point numbers.

Kinds are a higher-level classification system for types in Haskell.
Every type in Haskell has a kind, and this kind represents the "type of types".

The notation for kinds in Haskell uses an asterisk `*` to represent the most basic kind, which corresponds to concrete types.
You can check the kind of any type by using the `:kind` command in GHCi.

The kind of `Int` is `*`, indicating that `Int` is a concrete type.
```haskell
Int :: *
```

Similarly, the kind of `Float` is `*`, signifying that it is also a concrete type.
```haskell
Float :: *
```

However, Haskell's type system can go beyond basic types and deal with more complex constructs.
For instance, type classes introduce a kind called `Constraint`, denoted as `=>`.
The kind Constraint is used to represent constraints on types and is commonly encountered when defining type classes and their instances.

The kind of `Num` is `* -> Constraint`, showing that `Num` is a type class that takes a type (like `Int`) as an argument.
```haskell
Num :: * -> Constraint
```

When `Num` is applied to `Int`, it becomes a constraint on the `Int` type, indicating that `Int` is an instance of the `Num` type class.
```haskell
Num Int :: Constraint
```
Another example of a kind in Haskell is the kind of unary type constructors `* -> *`.
These unary type constructors are similar to generic types in other languages because they accept a type parameter and produce a new type.
The signature `* -> *` signifies that the type constructor transforms one concrete type into another concrete type.

An example of a trivial type constructor `Item` is shown below.

```haskell
data Item a = MkItem a

Item        :: * -> *
Item Int    :: *
Item String :: *
```

In this definition, `Item` is a unary type constructor because it takes a type parameter `a` and produces a new type `Item a`, where `a` can be any concrete type.

A commonly used example of a unary type constructor in Haskell is the `Maybe` type.
`Maybe` is used for representing optional values and handling potentially missing or undefined data in a type-safe manner.
Its definition is as follows:

```haskell
data Maybe a = Nothing | Just a

Maybe :: * -> *
```

Haskell type constructors can be parameterized with more than one type variable, as seen in the case of binary type constructors.
An example of a binary type constructor is the `Either` data type:

```haskell
data Either a b = Left a | Right b

Either            :: * -> * -> *
Either String     :: * -> *
Either String Int :: *
```

In this definition, `Either` is a binary type constructor because it requires two type parameters, `a` and `b`, to create a new type, `Either a b`.
The `Either` type is often used for scenarios where a value can have one of two possible types, such as representing success or failure or providing an alternative to error handling.
Conventionally, the `Left` value indicates error conditions, while the `Right` value signifies valid or correct values.


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

It is worth noting that type classes with a kind of `(* -> *) -> Constraint` can be applied to kinds like `* -> * -> *`.
An example of this is the `Functor` instance of `Either`:

```haskell
instance Functor (Either a) where
  fmap _ (Left x)   = Left x
  fmap f (Right x)  = Right (f x)
```

In the instance declaration above, we specify that, for any type `a`, `Either a` is an instance of the `Functor` type class.
This allows you to use the `fmap` function with `Either a`, providing a way to map over the second type argument of the `Either` type while keeping the first type argument fixed.


## Other languages

### Scala

scala

### Rust

rust


## Bibliography

:::bibtex
@inproceedings{10.1145/1238844.1238856,
  author = {Hudak, Paul and Hughes, John and Peyton Jones, Simon and Wadler, Philip},
  title  = {A History of Haskell: Being Lazy with Class},
  year   = {2007},
  isbn   = {9781595937667},
  publisher = {Association for Computing Machinery},
  address = {New York, NY, USA},
  url    = {https://doi.org/10.1145/1238844.1238856},
  doi    = {10.1145/1238844.1238856},
  abstract = {This paper describes the history of Haskell, including its genesis and principles, technical contributions, implementations and tools, and applications and impact.},
  booktitle = {Proceedings of the Third ACM SIGPLAN Conference on History of Programming Languages},
  pages  = {12–1–12–55},
  location = {San Diego, California},
  series = {HOPL III}
}

@online{youtube,
  title  = {Escape from the ivory tower: the Haskell journey},
  date   = {2017},
  organization = {Youtube},
  author = {Simon {Peyton Jones}},
  url    = {https://youtu.be/re96UgMk6GQ},
}

@book{skinner2023effective,
  title  = {Effective Haskell: Solving Real-World Problems with Strongly Typed Functional Programming},
  author = {Skinner, R.},
  isbn   = {9781680509342},
  series = {Pragmatic Bookshelf},
  year   = {2023},
  publisher = {O'Reilly Media}
}
:::
