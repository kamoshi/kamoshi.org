---
title: Haskell molecules
date: 2024-01-11T21:21:41.263Z
slug: haskell-molecules
---

# Haskell molecules

---

## Atoms

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

## .

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

## Isotopes

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
