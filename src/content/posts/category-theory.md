---
title: Category theory, abstract algebra and friends
date: 2023-05-14T21:57:54+02:00
---

## Monoid

$$
(M, *)
$$

```haskell
class Monoid a where
  mempty :: a
  mappend :: a -> a -> a
```