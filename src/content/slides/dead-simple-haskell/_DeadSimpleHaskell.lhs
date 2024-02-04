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
() | Bool  | 1 + 2 = 3
H' | Bool  | 3 + 2 = 5

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

(A change of topic)

---

```haskell
data Box a
  = Has a
  | Empty
```

---

```haskell
class Mappable box where
  map' :: box a -> box b
```

---

```haskell
class Appliable box where
  wrap   :: a -> box a
  apply' :: box (a -> b) -> box a -> box b
```

---

```haskell
class Chainable box where
  chain :: box a -> (a -> box b) -> box b
```

-----

In fact it all already exists.

![This was all dream](/static/content/slides/haskell-molecules/_cave.jpg)

---

What's a `Mappable`?

---

![It's a Functor](/static/content/slides/haskell-molecules/scoobydoo.jpg)

---

What's a `Appliable`?

---

![It's a Functor](/static/content/slides/haskell-molecules/scoobydoo.jpg)

---

What's a `Chainable`?

---

![It's a Functor](/static/content/slides/haskell-molecules/scoobydoo.jpg)

---

## Fun with monadic parsing


---

## More here:

- **Learn You a Haskell for Great Good!** by Miran Lipovaca
- **Programming in Haskell - 2nd Edition** by Graham Hutton
- **Effective Haskell** by Rebecca Skinner

