---
title: "Type Challenges"
---

## 4・Pick

> Implement the built-in `Pick<T, K>` generic without using it. Constructs a type by picking the set of properties `K` from `T`.

```ts
type MyPick<T, K extends keyof T> = {
  [key in K]: k extends keyof T ? T[key] : never;
}
```


## 7・Readonly

> Implement the built-in `Readonly<T>` generic without using it. Constructs a type with all properties of T set to readonly, meaning the properties of the constructed type cannot be reassigned.

```ts
type MyReadonly<T> = {
  readonly [key in keyof T]: T[key];
}
```


## 11・Tuple to Object

> Given an array, transform it into an object type and the key/value must be in the provided array.

```ts
type TupleToObject<T extends readonly (string | number)[]> = {
  [val in T[number]]: val;
}
```
