---
title: Dead simple Haskell
date: 2024-01-11T21:21:41.263Z
slug: dead-simple-haskell
animate: true
---

# Dead simple Haskell

![Haskell logo](/static/content/slides/haskell-molecules/haskell.png)

-----

## Haskell molecules

-----

### Atoms

![atoms](/static/content/slides/haskell-molecules/atoms.png)

---

![atoms](/static/content/slides/haskell-molecules/atoms.png)

```haskell
data H = H -- hydrogen
data O = O -- oxygen
data C = C -- carbon
```

-----

### Molecules

![Atoms](/static/content/slides/haskell-molecules/molecules.png)

---

![Atoms](/static/content/slides/haskell-molecules/molecules.png)

```haskell
type H₂O = (H, O, H) -- water
type O₂  = (O, O)    -- oxygen (gas)
type CO₂ = (O, C, O) -- carbon dioxide
```

-----

### Reactions

![Magic](/static/content/slides/haskell-molecules/magic.png)

---

![More magic](/static/content/slides/haskell-molecules/more-magic.png)

---

![More magic](/static/content/slides/haskell-molecules/more-magic.png)

```haskell ignore
makeWater :: H -> H -> O -> H₂O
```
```haskell ignore
makeOxygen :: O -> O -> O₂
```
```haskell ignore
burnOxygen :: C -> O₂ -> CO₂
```

---

```haskell
makeWater :: H -> H -> O -> H₂O
```
```haskell
makeWater h1 h2 o = (h1, o, h2)
```
~

```haskell
makeOxygen :: O -> O -> O₂
```
```haskell
makeOxygen o1 o2 = (o1, o2)
```
~

```haskell
burnOxygen :: C -> O₂ -> CO₂
```
```haskell
burnOxygen c (o1, o2) = (o1, c, o2)
```

-----

### `.` `$`

![Plumbing](/static/content/slides/haskell-molecules/plumbing.png)

---

![Partial application](/static/content/slides/haskell-molecules/partial.png)

```haskell ignore
λ> :type makeOxygen
makeOxygen :: O -> O -> O₂
```
```haskell ignore
λ> :type makeOxygen O
makeOxygen O :: O -> O₂
```
```haskell ignore
λ> :type makeOxygen O O
makeOxygen O O :: O₂
```

---

![Combustion](/static/content/slides/haskell-molecules/combustion.png)

```haskell ignore
λ> :type burnOxygen
burnOxygen :: C -> O₂ -> CO₂
```
```haskell ignore
λ> :type burnOxygen C
burnOxygen C :: O₂ -> CO₂
```
```haskell ignore
λ> :type burnOxygen C (O, O)
burnOxygen C O₂ :: CO₂
```

---


![Two functions](/static/content/slides/haskell-molecules/two-fs.png)

```haskell
f1 :: O -> O₂
f1 = makeOxygen O

f2 :: O₂ -> CO₂
f2 = burnOxygen C
```

---

![Composition](/static/content/slides/haskell-molecules/composition.png)


```haskell
f3 = f2 . f1
```

---

![Composition2](/static/content/slides/haskell-molecules/composition2.png)

```haskell
f3 :: O -> CO₂
```

```haskell ignore
f1 :: { O -> O₂ }
f2 :: [ O₂ -> CO₂ ]
f3 :: {O -> [ O₂ } -> CO₂ ]
f3 :: O -> CO₂
```

---

```haskell ignore
f3' o = f2 . f1 o
```

```scala
Diagnostics:
1. • Couldn't match type: (O, O)
                    with: a -> O₂
     Expected: a -> O₂
       Actual: O₂
   • Possible cause: ‘f1’ is applied to too many arguments
     In the second argument of ‘(.)’, namely ‘f1 o’
     In the expression: f2 . f1 o
     In an equation for ‘f3'’: f3' o = f2 . f1 o
   • Relevant bindings include
       f3' :: O -> a -> CO₂
         (bound at Script.hs:112:1) [-Wdeferred-type-errors]
```

---

![Composition3](/static/content/slides/haskell-molecules/composition3.png)

```haskell ignore
f3' o = f2 . f1 o
```

---

![Funnel](/static/content/slides/haskell-molecules/funnel.png)

```haskell ignore
f3' o = f2 . f1 $ o

-- OR

f3' o = f2 $ f1 o
```

---

```haskell ignore
infixr 9 .
(.) :: (b -> c) -> (a -> b) -> (a -> c)
(f . g) x = f (g x)

infixr 0 $
($) :: (a -> b) -> a -> b
f $ x = f x
```

-----

## Isotopes

![Heavy water](/static/content/slides/haskell-molecules/heavy-water.gif)

---

![Isotopes](/static/content/slides/haskell-molecules/isotopes.png)

---

```haskell
data H'
  = H¹-- protium
  | H²-- deuterium
  | H³-- tritium

type H2O' = (H', O, H') -- water
```

```haskell
makeWater' :: H' -> H' -> O -> H2O'
makeWater' h1 h2 o = (h1, o, h2)
```

---

### Algebra

How many values inhibit each of these?

Type       | Possibilities
---------- | -------------
`()`       | ?
`O`        | ?
`H'`       | ?
`(H', H')` | ?

---

`()`

---

`()`

1

`()`

---

`O`

---

`O`

1

`O`

---

`H'`

---

`H'`

3

`H¹` `H²` `H³`

---

`(H', H')`

---

`(H', H')`

9

`(H¹, H¹)` `(H¹, H²)` `(H¹, H³)`

`(H², H¹)` `(H², H²)` `(H², H³)`

`(H³, H¹)` `(H³, H²)` `(H³, H³)`

---

`(H', Bool)`

---

`(H', Bool)`

6

`(H¹, True)` `(H¹, False)`

`(H², True)` `(H², False)`

`(H³, True)` `(H³, False)`

---

Type       | Inhibitants
---------- | -----------
`()`       | 1
`Bool`     | 2
`H'`       | 3
`A`        | a
`B`        | b
`(A, B)`   | a × b
`A` \| `B` | a + b

---

Product types Π

Type       | Inhibitants
---------- | -----------
()         | 1
Bool       | 2
((), Bool) | 1 × 2 = 2
(H', Bool) | 3 × 2 = 6

```haskell ignore
data Pair a b = Pair a b
```

---

```haskell ignore
type Pair a b = (a, b)
```

```haskell ignore
data Pair a b = Pair a b
```

```haskell ignore
data Pair a b = Pair
  { fieldA :: a
  , fieldB :: b
  }
```

---

Sum types Σ

Type       | Inhibitants
---------- | -----------
()         | 1
Bool       | 2
() \| Bool | 1 + 2 = 3
H' \| Bool | 3 + 2 = 5

```haskell ignore
data Either a b = Left a | Right b
```

---

Why not both?

---

Why not both?

```haskell ignore
data Crazy a b
  = CrazyA a
  | CrazyB b
  | Both a b
  | Neither
```

---

```haskell ignore
data Crazy a b
  = CrazyA a
  | CrazyB b
  | Both a b
  | Neither
```

Member     | Inhibitants
---------- | -----------
`CrazyA a` | a
`CrazyB b` | b
`Both a b` | a × b
`Neither`  | 1
Σ          | a + b + a × b + 1

---

Can there be a "0"?

---

Can there be a "0"?

Yes

```haskell ignore
Crazy Void ()
  = CrazyA Void
  | CrazyB ()
  | Both Void ()
  | Neither
```

Σ = 0 + 1 + 0 × 1 + 1 = 2

---

How do lists fit into this?

---

How do lists fit into this?

```haskell
data List a
  = Nil
  | Cons a (List a)
```
```haskell
-- [1, 2, 3] or 1 : 2 : 3 []
list1 = Cons 1 (Cons 2 (Cons 3 Nil))
-- []
list2 = Nil
```

---

```haskell ignore
data List a
  = Nil
  | Cons a (List a)
```

x = 1 + a * x

---

```haskell ignore
data List a
  = Nil
  | Cons a (List a)
```

```
x = 1 + a * x
x = 1 + a * (1 + a * x)
x = 1 + a * (1 + a * (1 + a * (...)))
```

---

```haskell ignore
data List a
  = Nil
  | Cons a (List a)
```

```
x = 1 + a * x
x = 1 + a * (1 + a * x)
x = 1 + a * (1 + a * (1 + a * (...)))
```

```
x = 1 + a + a² + a³ + a⁴ + a⁵ + ...
```

---

```haskell ignore
data List a
  = Nil
  | Cons a (List a)
```

```
x = 1 + a * x
x = 1 + a * (1 + a * x)
x = 1 + a * (1 + a * (1 + a * (...)))
```

```
x = 1 + a + a² + a³ + a⁴ + a⁵ + ...
```
```haskell ignore
data List a = () | (a) | (a, a) | (a, a, a) | ...
```

-----

### Traits

![Neon colors](/static/content/slides/haskell-molecules/neon.png)

---

Element name | Color
------------ | ------
Helium       | orange
Neon         | red
Argon        | lavender
Krypton      | white
Xenon        | blue
Radon        | red

---

```haskell
data He = He -- Helium
data Ne = Ne -- Neon
data Ar = Ar -- Argon
data Kr = Kr -- Krypton
data Xe = Xe -- Xenon
data Rn = Rn -- Radon
```

---

How to convert noble gases to color?

```haskell ignore
toColor :: ? -> String
```

---

What about allowing everything in?

```haskell ignore
toColor :: a -> String
toColor a
  | a == He   = "orange"
  | a == Ne   = "red"
  | otherwise = undefined

-- ???
_ = toColor "anything in"
_ = toColor 1234
```

---

What about treating noble gases as a sum type?

```haskell ignore
data Noble
  = He
  | Ne
  | Ar
  | Kr
  | Xe
  | Rn
```

---

`a -> String`

^

???

^

`Noble -> String`

---

```haskell
class Noble a where
  toColor :: a -> String
```

~

```haskell
instance Noble He where
  toColor _ = "orange"

instance Noble Ne where
  toColor _ = "red"

instance Noble Ar where
  toColor _ = "lavender"
```

---

Works:

```haskell ignore
λ> toColor He
"orange"
λ> toColor Ne
"red"
```

Doesn't work:

```haskell ignore
λ> toColor C

<interactive>:6:1: error:
    • No instance for (Noble C) arising from a use of ‘toColor’
    • In the expression: toColor C
      In an equation for ‘it’: it = toColor C
```

---

```haskell
mixNoble :: (Noble n1, Noble n2) => n1 -> n2 -> String
mixNoble n1 n2 = toColor n1 <> "-" <> toColor n2
```

```haskell ignore
λ> mixNoble He Ne
"orange-red"
```

-----

## Shrodinger's cat

![Cat in a box](/static/content/slides/haskell-molecules/cat.png)

---

![Cat in a box](/static/content/slides/haskell-molecules/cat.png)

```haskell
data Box a
  = Has a
  | Empty
  deriving Show
```

---

![Map](/static/content/slides/haskell-molecules/map.png)

```haskell
data Cat = Cat String deriving Show
data Dog = Dog String deriving Show
```

---

![Map](/static/content/slides/haskell-molecules/map.png)

```haskell
class Mappable box where
  map' :: (a -> b) -> box a -> box b
```

---

```haskell ignore
class Mappable box where
  map' :: (a -> b) -> box a -> box b
```

```haskell
instance Mappable Box where
  map' _ Empty   = Empty
  map' f (Has a) = Has (f a)
```
```haskell
instance Mappable [] where
  map' _ [] = []
  map' f xs = [f x | x <- xs]
```

---

```haskell ignore
instance Mappable [] where
  map' _ [] = []
  map' f xs = [f x | x <- xs]
```

![Cat list](/static/content/slides/haskell-molecules/cat-list.png)

---

![Cat list](/static/content/slides/haskell-molecules/cat-list.png)

```haskell ignore
λ> a = Has $ Cat "cat"
λ> :type a
a :: Box Cat
λ> :type map' toDog a
map' toDog a :: Box Dog
λ> :type map toDog [Cat "a", Cat "b"]
map toDog [Cat "a", Cat "b"] :: [Dog]
```

---

```haskell
toDog :: Cat -> Dog
toDog (Cat name) = Dog name
```

```haskell
convertAllCats :: Mappable box => box Cat -> box Dog
convertAllCats cs = map' toDog cs
```

---

Any "box" can go in 👍

```haskell ignore
convertAllCats :: Mappable box => box Cat -> box Dog
convertAllCats cs = map' toDog cs
```

```haskell ignore
λ> convertAllCats [Cat "a", Cat "b"]
[Dog "a",Dog "b"]
λ> convertAllCats $ Has $ Cat "a"
Has (Dog "a")
```

-----

![Cat merge](/static/content/slides/haskell-molecules/cat-merge.png)

---

![Cat merge](/static/content/slides/haskell-molecules/cat-merge.png)

```haskell
merge :: Cat -> Cat -> Dog
merge (Cat c1)  (Cat c2) = Dog $ c1 <> " & " <> c2
```

---

```haskell
catA = Cat "Meow"
catB = Cat "Nyaa"
```

```haskell ignore
λ> merge catA catB
Dog "Meow & Nyaa"
```

![Cat merge](/static/content/slides/haskell-molecules/merge.png)

---

```haskell
boxA  = Has $ Cat "Meow" :: Box Cat
boxB  = Has $ Cat "Nyaa" :: Box Cat
empty = Empty            :: Box Cat
```

![Cat merge](/static/content/slides/haskell-molecules/merge-box.png)

---

```haskell ignore
boxA  = Has $ Cat "Meow" :: Box Cat
boxB  = Has $ Cat "Nyaa" :: Box Cat
empty = Empty            :: Box Cat
```

![Cat merge](/static/content/slides/haskell-molecules/merge-box.png)

```haskell ignore
λ> merge boxA catB

<interactive>:7:7: error:
    • Couldn't match expected type ‘Cat’ with actual type ‘Box Cat’
    • In the first argument of ‘merge’, namely ‘boxA’
      In the expression: merge boxA catB
      In an equation for ‘it’: it = merge boxA catB
```

---

```haskell ignore
boxA  = Has $ Cat "Meow" :: Box Cat
boxB  = Has $ Cat "Nyaa" :: Box Cat
empty = Empty            :: Box Cat
```

![Cat merge](/static/content/slides/haskell-molecules/merge-prim.png)

```haskell
merge' :: Box Cat -> Box Cat -> Box Dog
merge' Empty _ = Empty
merge' _ Empty = Empty
merge' (Has c1) (Has c2) = Has $ merge c1 c2
```

---

```haskell ignore
merge' :: Box Cat -> Box Cat -> Box Dog
merge' Empty _ = Empty
merge' _ Empty = Empty
merge' (Has c1) (Has c2) = merge c1 c2
```

![Cat merge](/static/content/slides/haskell-molecules/merge-prim.png)

```haskell ignore
λ> merge' boxA boxB
Has (Dog "Meow & Nyaa")
λ> merge' boxA (Has catB)
Has (Dog "Meow & Nyaa")
λ> merge' boxA empty
Empty
```

---

```haskell
class Appliable box where
  wrap   :: a -> box a
  apply' :: box (a -> b) -> box a -> box b
```

![Wrap apply](/static/content/slides/haskell-molecules/wrap-apply.png)

---

```haskell ignore
class Appliable box where
  wrap   :: a -> box a
  apply' :: box (a -> b) -> box a -> box b
```

```haskell
instance Appliable Box where
  wrap = Has
  apply' Empty _ = Empty
  apply' _ Empty = Empty
  apply' (Has f) (Has a) = Has $ f a

instance Appliable [] where
  wrap x = [x]
  apply' [] _ = []
  apply' _ [] = []
  apply' fs xs = [f x | f <- fs, x <- xs]
```

---


```haskell ignore
instance Appliable Box where
  wrap = Has
  apply' Empty _ = Empty
  apply' _ Empty = Empty
  apply' (Has f) (Has a) = Has $ f a

instance Appliable [] where
  wrap x = [x]
  apply' [] _ = []
  apply' _ [] = []
  apply' fs xs = [f x | f <- fs, x <- xs]
```

```haskell ignore
λ> apply' (apply' (wrap merge) boxA) boxB
Has (Dog "Meow & Nyaa")
λ> apply' (apply' (wrap merge) boxA) empty
Empty
```

---

```haskell ignore
instance Appliable Box where
  wrap = Has
  apply' Empty _ = Empty
  apply' _ Empty = Empty
  apply' (Has f) (Has a) = Has $ f a

instance Appliable [] where
  wrap x = [x]
  apply' [] _ = []
  apply' _ [] = []
  apply' fs xs = [f x | f <- fs, x <- xs]
```

```haskell ignore
λ> apply' (apply' (wrap merge) [Cat "A"]) [Cat "B1", Cat "B2"]
[Dog "A & B1",Dog "A & B2"]
λ> apply' (apply' (wrap merge) [Cat "A"]) []
[]
```

---


```haskell ignore
λ> apply' (apply' (wrap merge) [Cat "A"]) [Cat "B1", Cat "B2"]
[Dog "A & B1",Dog "A & B2"]
λ> apply' (apply' (wrap merge) [Cat "A"]) []
[]
```

```haskell ignore
λ> wrap merge `apply'` [Cat "A"] `apply'` [Cat "B1", Cat "B2"]
[Dog "A & B1",Dog "A & B2"]
λ> wrap merge `apply'` [Cat "A"] `apply'` []
[]
```

---

```haskell ignore
λ> wrap merge `apply'` [Cat "A"] `apply'` [Cat "B1", Cat "B2"]
[Dog "A & B1",Dog "A & B2"]
λ> wrap merge `apply'` [Cat "A"] `apply'` []
[]
```

```haskell ignore
λ> wrap merge `apply'` boxA `apply'` boxB
Has (Dog "Meow & Nyaa")
λ> wrap merge `apply'` boxA `apply'` empty
Empty
```

![Wrap apply](/static/content/slides/haskell-molecules/wrap-apply.png)

---

![Wrap apply](/static/content/slides/haskell-molecules/wrap-apply.png)

```haskell
merge4 :: Cat -> Cat -> Cat -> Cat -> Dog
merge4 (Cat a) (Cat b) (Cat c) (Cat d) = Dog $ a <> b <> c <> d
```

```haskell ignore
λ> wrap merge4 `apply'` boxA `apply'` boxB `apply'` boxA `apply'` boxB
Has (Dog "MeowNyaaMeowNyaa")
λ> wrap merge4 `apply'` boxA `apply'` boxB `apply'` empty `apply'` boxB
Empty
```

-----

```haskell
killOrSave :: Cat -> Box Cat
killOrSave cat@(Cat name) = case name of
  "Meow" -> Empty
  _      -> Has cat
```

<small>(no animals were harmed in the making of this slideshow)</small>

---

```haskell ignore
killOrSave :: Cat -> Box Cat
killOrSave cat@(Cat name) = case name of
  "Meow" -> Empty
  _      -> Has cat
```

![Kill](/static/content/slides/haskell-molecules/kill.png)

---

![Kill](/static/content/slides/haskell-molecules/kill.png)

```haskell ignore
λ> wrap killOrSave `apply'` Has (Cat "Meow")
Has Empty
λ> wrap killOrSave `apply'` Has (Cat "Nyaa")
Has (Has (Cat "Nyaa"))
λ> wrap killOrSave `apply'` Empty
Empty
```

---

```haskell
class Chainable box where
  chain :: box a -> (a -> box b) -> box b
```

---

```haskell ignore
class Chainable box where
  chain :: box a -> (a -> box b) -> box b
```

```haskell
instance Chainable Box where
  chain Empty _ = Empty
  chain (Has a) f = f a

instance Chainable [] where
  chain [] _ = []
  chain xs f = [x' | x <- xs, x' <- f x]
```

---

```haskell ignore
instance Chainable Box where
  chain Empty _ = Empty
  chain (Has a) f = f a

instance Chainable [] where
  chain [] _ = []
  chain xs f = [x' | x <- xs, x' <- f x]
```

![Save-kill](/static/content/slides/haskell-molecules/save-kill.png)

---

![Save-kill](/static/content/slides/haskell-molecules/save-kill.png)

```haskell
kill :: Cat -> Box Cat
kill _ = Empty

save :: Cat -> Box Cat
save = Has
```

---

![Save-kill](/static/content/slides/haskell-molecules/save-kill.png)

```haskell ignore
λ> boxA `chain` save `chain` save `chain` save
Has (Cat "Meow")
λ> boxA `chain` save `chain` kill `chain` save
Empty
```

---

```haskell
kill' :: Cat -> [Cat]
kill' _ = []

save' :: Cat -> [Cat]
save' c = [c]

clone :: Cat -> [Cat]
clone (Cat c) = [Cat $ c <> "L" , Cat $ c <> "R"]
```

---

```haskell ignore
kill' :: Cat -> [Cat]
kill' _ = []

save' :: Cat -> [Cat]
save' c = [c]

clone :: Cat -> [Cat]
clone (Cat c) = [Cat $ c <> "L" , Cat $ c <> "R"]
```

```haskell ignore
λ> [catA] `chain` save' `chain` save' `chain` save'
[Cat "Meow"]
λ> [catA] `chain` save' `chain` clone `chain` save'
[Cat "MeowL",Cat "MeowR"]
λ> [catA] `chain` more' `chain` clone `chain` save'
[Cat "MeowLL",Cat "MeowLR",Cat "MeowRL",Cat "MeowRR"]
λ> [catA] `chain` clone `chain` clone `chain` kill'
[]
```

---

```haskell ignore
λ> [catA] `chain` save' `chain` save' `chain` save'
[Cat "Meow"]
λ> [catA] `chain` save' `chain` clone `chain` save'
[Cat "MeowL",Cat "MeowR"]
λ> [catA] `chain` more' `chain` clone `chain` save'
[Cat "MeowLL",Cat "MeowLR",Cat "MeowRL",Cat "MeowRR"]
λ> [catA] `chain` clone `chain` clone `chain` kill'
[]
```

![Chain list](/static/content/slides/haskell-molecules/chain-list.png)

---

![Callbacks](/static/content/slides/haskell-molecules/callbacks.jpg)

-----

![This was all dream](/static/content/slides/haskell-molecules/cave.png)

---

What's a `Mappable`?

---

It's a `Functor`

![It's a Functor](/static/content/slides/haskell-molecules/its-functor.png)

---

It's a `Functor`

```haskell ignore
class Mappable box where
  map' :: (a -> b) -> box a -> box b
```

```haskell ignore
class Functor f where
  fmap :: (a -> b) -> f a -> f b
  (<$) :: a -> f b -> f a
```

---

It's a `Functor`

```haskell ignore
λ> map' (+1) []
[]
λ> map' (+1) [1, 2, 3]
[2,3,4]
λ> map' (*3) [1, 2, 3]
[3,6,9]
```

```haskell ignore
λ> fmap (+1) []
[]
λ> fmap (+1) [1, 2, 3]
[2,3,4]
λ> fmap (*3) [1, 2, 3]
[3,6,9]
λ> 1 <$ [1, 2, 3]
[1,1,1]
```

---

It's a `Functor`

![Tardis](/static/content/slides/haskell-molecules/tardis.jpg)

---

It's a `Functor`

![Tardis inside](/static/content/slides/haskell-molecules/tardis-inside.jpg)

---

```haskell ignore
main :: IO ()
main = print "Hello World!"
```

![Tardis inside](/static/content/slides/haskell-molecules/io.png)

---

What's a `Appliable`?

---

It's an Applicative

![It's an applicative](/static/content/slides/haskell-molecules/its-applicative.png)

---

It's an Applicative

```haskell ignore
class Appliable box where
  wrap   :: a -> box a
  apply' :: box (a -> b) -> box a -> box b
```

```haskell ignore
class (Functor f) => Applicative f where
  pure  :: a -> f a
  (<*>) :: f (a -> b) -> f a -> f b
```

---

It's an Applicative

```haskell ignore
λ> wrap merge `apply'` [catA] `apply'` []
[]
λ> wrap merge `apply'` [catA] `apply'` [catB]
[Dog "Meow & Nyaa"]
λ> wrap merge `apply'` [catA] `apply'` [catB, Cat "C"]
[Dog "Meow & Nyaa",Dog "Meow & C"]
```

```haskell ignore
λ> pure merge <*> [catA] <*> []
[]
λ> pure merge <*> [catA] <*> [catB]
[Dog "Meow & Nyaa"]
λ> pure merge <*> [catA] <*> [catB, Cat "C"]
[Dog "Meow & Nyaa",Dog "Meow & C"]
```

---

```haskell ignore
λ> pure merge <*> [catA] <*> []
[]
λ> pure merge <*> [catA] <*> [catB]
[Dog "Meow & Nyaa"]
λ> pure merge <*> [catA] <*> [catB, Cat "C"]
[Dog "Meow & Nyaa",Dog "Meow & C"]
```

```haskell ignore
λ> merge <$> [catA] <*> []
[]
λ> merge <$> [catA] <*> [catB]
[Dog "Meow & Nyaa"]
λ> merge <$> [catA] <*> [catB, Cat "C"]
[Dog "Meow & Nyaa",Dog "Meow & C"]
```

```haskell ignore
merge :: Cat -> Cat -> Dog

_ = merge catA catB
```

---

What's a `Chainable`?

---

It's a monad

![It's a monad](/static/content/slides/haskell-molecules/its-monad.png)

---

It's a monad

```haskell ignore
class Chainable box where
  chain :: box a -> (a -> box b) -> box b
```

```haskell ignore
class Applicative m => Monad m where
  (>>=) :: m a -> (a -> m b) -> m b

  (>>) :: m a -> m b -> m b
  m >> k = m >>= \_ -> k

  return :: a -> m a
  return = pure
```

---

It's a monad

```haskell ignore
λ> [catA] `chain` save' `chain` save' `chain` save'
[Cat "Meow"]
λ> [catA] `chain` save' `chain` clone `chain` save'
[Cat "MeowL",Cat "MeowR"]
λ> [catA] `chain` save' `chain` clone `chain` kill'
[]
```

```haskell ignore
λ> [catA] >>= save' >>= save' >>= save'
[Cat "Meow"]
λ> [catA] >>= save' >>= clone >>= save'
[Cat "MeowL",Cat "MeowR"]
λ> [catA] >>= save' >>= clone >>= kill'
[]
```

---

![Just one more typeclass bro](/static/content/slides/haskell-molecules/pepe.png)

-----

## More here:

- **Learn You a Haskell for Great Good!** by Miran Lipovaca
- **Programming in Haskell - 2nd Edition** by Graham Hutton
- **Effective Haskell** by Rebecca Skinner

