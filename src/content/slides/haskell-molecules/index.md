---
title: Haskell molecules
date: 2024-01-11T21:21:41.263Z
slug: haskell-molecules
---

# ??????

---

## Haskell molecules

---

### Atoms

(atom drawing here)

---

```haskell
data H = H -- hydrogen
data O = O -- oxygen
data C = C -- carbon
```

---

```haskell
type H₂O = (H, O, H) -- water
type O₂  = (O, O)    -- oxygen (gas)
type CO₂ = (O, C, O) -- carbon dioxide
```

---

```haskell
makeWater  :: H -> H -> O -> H₂O
makeOxygen :: O -> O -> O₂
burnOxygen :: C -> O₂ -> CO₂
```

```haskell
makeWater h1 h2 o     = (h1, o, h2)
makeOxygen o1 o2      = (o1, o2)
burnOxygen c (o1, o2) = (o1, c, o2)
```

---

### .

(pipe drawing here)

---

```haskell
-- O -> O -> O₂
_ = makeOxygen

-- O -> O₂
_ = makeOxygen O

-- O₂
_ = makeOxygen O O
```
---

```haskell
-- C -> O₂ -> CO₂
_ = burnOxygen

-- O₂ -> CO₂
_ = burnOxygen C

-- CO₂
_ = burnOxygen C (O, O)
```

---

```haskell
f1 :: O -> O₂
f1 = makeOxygen O

f2 :: O₂ -> CO₂
f2 = burnOxygen C
```

---

```haskell
f3 = f2 . f1
```

---

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

```haskell ignore
f3' o = f2 . f1 $ o

-- OR

f3' o = f2 $ f1 o
```

---

### Isotopes

---

```haskell
data H'
  = H¹ -- protium
  | H² -- deuterium
  | H³ -- tritium

type H2O' = (H', O, H') -- water
```

```haskell
makeWater' :: H' -> H' -> O -> H2O'
makeWater' a b c = (a, c, b)
```

---

### Algebra

- Sum type
- Product type

---

### Traits

Noble gases are often used in fluorescent lighting and discharge lamps.

![Neon colors](/static/content/slides/haskell-molecules/neon.png)

---

Element | Color
------- | ------
Helium  | orange
Neon    | ???
Argon   | lavender
Krypton | white
Xenon   | blue
Radon   | red

---

A function which would map noble gases to color?

```haskell ignore
toColor :: ? -> String
```

---

What about allowing everything "in"?

```haskell ignore
toColor :: a -> String
toColor a
  | a == "a" = "orange"

-- ???
_ = toColor "anything in"
_ = toColor 1234
```

---

What about treating noble gases as a sum type?

---

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

---

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



