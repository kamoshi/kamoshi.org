---
title: "Rust introduction"
date: 2023-02-10T13:51:58+01:00
---
# Introduction to...

![Rust](/static/slides/rust-introduction/rust.png)

-----

## Introduction

-----

### Types

---

#### Primitives

- signed ints: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- unsigned ints: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- floating point: `f32`, `f64`
- char: Unicode scalar
- bool: `true`, `false`
- unit: `()`

---

| Length  | Signed | Unsigned |
| ------- | ------ | -------- |
| 8-bit   | i8     | u8       |
| 16-bit  | i16    | u16      |
| 32-bit  | i32    | u32      |
| 64-bit  | i64    | u64      |
| 128-bit |	i128   | u128     |
| arch    | isize  | usize    |

---

#### Compound

| Name  | Type        | Example value |
| ----- | ----------- | ------------- |
| Array | `[3; i32]`  | `[1, 2, 3]`   |
| Tuple | `(i32, bool)` | `(1, true)` |

---

#### Enum

```rs
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}
```

---

```rs
enum Coin {
    Penny,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}
```

---

#### Option & Result

```rs
enum Option<T> {
    None,
    Some(T),
}

enum Result<T, E> {
   Ok(T),
   Err(E),
}
```

---

#### Struct

```rs [1-4|6-10|12-19]
struct Container {
    a: i32,
    b: i32,
}

impl Container {
    fn sum_ab(&self) -> i32 {
        self.a + self.b
    }
}

fn main() {
    let container = Container {
        a: 10,
        b: 15,
    };

    let sum = container.sum_ab();
}
```

---

#### Vector
A contiguous growable array type, written as `Vec<T>`, short for ‘vector’.

---

```rs [1-3|5-6|8-9|11-12]
let mut vec = Vec::new();
vec.push(1);
vec.push(2);

assert_eq!(vec.len(), 2);
assert_eq!(vec[0], 1);

assert_eq!(vec.pop(), Some(2));
assert_eq!(vec.len(), 1);

vec[0] = 7;
assert_eq!(vec[0], 7);
```

---

The `vec!` macro is provided for convenient initialization:
```rs
let mut vec1 = vec![1, 2, 3];
vec1.push(4);
let vec2 = Vec::from([1, 2, 3, 4]);
assert_eq!(vec1, vec2);
```

-----

### Control flow

---

#### if
```rs
fn main() {
    let number = 6;

    if number % 4 == 0 {
        println!("number is divisible by 4");
    } else if number % 3 == 0 {
        println!("number is divisible by 3");
    } else if number % 2 == 0 {
        println!("number is divisible by 2");
    } else {
        println!("number is not divisible by 4, 3, or 2");
    }
}
```

---

```rs
fn main() {
    let condition = true;
    let number = if condition { 5 } else { 6 };

    println!("The value of number is: {number}");
}
```

---

#### loop

```rs
fn main() {
    loop {
        println!("again!");
    }
}
```

---

```rs
fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;

        if counter == 10 {
            break counter * 2;
        }
    };

    println!("The result is {result}");
}
```

---

### while

```rs
fn main() {
    let mut number = 3;

    while number != 0 {
        println!("{number}!");

        number -= 1;
    }

    println!("LIFTOFF!!!");
}
```

---

### for

```rs
fn main() {
    let a = [10, 20, 30, 40, 50];

    for element in a {
        println!("the value is: {element}");
    }
}
```

-----

### Iterators

JS:
```js
myList
  .filter((t) => t !== "Filter me out")
  .forEach(console.log);
```

Rust:
```rs
myList.into_iter()
  .filter(|v| v != "Filter me out")
  .for_each(|v| println!("{}", v));
```

---

| Funtion    | What it does |
| ---------- | ------------ |
| .take(n)   | reduces an iterator to it’s first n elements |
| .skip(n)   | skips the first n elements |
| .cloned()  | clones each element in the iterator |
| .enumerate | turns an iterator over elements t to an iterator over elements `(t: T, idx: usize)` |
| .cycle     | loops the iterator infinitely |
| .rev       | reverses an iterator |

-----

### Attributes

---

```rs [1-4|7-11|13-15|17-21]
// A function marked as a unit test
#[test]
fn test_foo() {
    /* ... */
}

// A conditionally-compiled module
#[cfg(target_os = "linux")]
mod bar {
    /* ... */
}

// A lint attribute used to suppress a warning/error
#[allow(non_camel_case_types)]
type int8_t = i8;

// Inline function, when compiling to machine code
#[inline(always)]
fn siema() {
    /* ... */
}
```

-----

### Ownership

---

```rs [1-4|6-8|10-12|14-16|19-20|22-23|25-26|28-31]
struct Container {
    a: i32,
    b: i32,
}

fn consume(container: Container) -> i32 {
    container.a + container.b
}

fn borrow(container: &Container) -> i32 {
    container.a + container.b
}

fn borrow_mut(container: &mut Container) -> i32 {
    container.a + container.b
}

fn main() {
    let c1 = Container { a: 10, b: 15 };
    let c2 = Container { ..c1 };

    let sum1 = consume(c1);
    // let sum1 = consume(c1); // illegal

    let sum2 = borrow(&c2);
    let sum2 = borrow(&c2); // ok

    let mut c3 = Container { ..c2 };

    let sum3 = borrow_mut(&mut c3);
    let sum3 = borrow_mut(&mut c3); // ok
}
```

---

#### Borrow checker

![compiler error](/static/slides/rust-introduction/Screenshot_20230210_174448.png)

---

#### Mutable borrow

```rs [4|6-8|10-14|16-19]
fn main() {
    // --snip--

    let mut c3 = Container { ..c2 };

    // ok
    let ref1 = &c3;
    let ref2 = &c3;

    // ok
    let ref1 = &mut c3;
    ref1.a;
    // compiler knows the first borrow can be dropped here
    let ref2 = &mut c3;

    // illegal
    let ref1 = &mut c3;
    let ref2 = &mut c3;
    ref1.a;
}
```

---

#### Borrow checker

![compiler error](/static/slides/rust-introduction/Screenshot_20230210_175828.png)

-----

### Lifetimes

---

```rs [1-3|5-9|11-14]
fn test(arr: &[i32]) -> impl Iterator<Item=&i32> {
    return  arr.iter().cycle()
}

fn main() { // works
    let x = vec![1, 2, 3];
    let x = test(&x);
    x.take(9).for_each(|i| print!("{i}") );
}

fn main() { // doesn't work
    let x = test(&vec![1, 2, 3]);
    x.take(9).for_each(|i| print!("{i}") );
}
```

---

![borrow cheecker](/static/slides/rust-introduction/Screenshot_20230222_180500.png)

---

```rs
fn borrow(c: &Container) {
    // -- snip --
}
```

---

```rs
fn borrow<'a>(c: &'a Container) {
    // -- snip -- 
}
```

---

```rs [1-17|4-8|11-15]
fn main() {
    let i = 3; // Lifetime for `i` starts. ────────────────┐
    //                                                     │
    { //                                                   │
        let borrow1 = &i; // `borrow1` lifetime starts. ──┐│
        //                                                ││
        println!("borrow1: {}", borrow1); //              ││
    } // `borrow1` ends. ─────────────────────────────────┘│
    //                                                     │
    //                                                     │
    { //                                                   │
        let borrow2 = &i; // `borrow2` lifetime starts. ──┐│
        //                                                ││
        println!("borrow2: {}", borrow2); //              ││
    } // `borrow2` ends. ─────────────────────────────────┘│
    //                                                     │
}   // Lifetime ends. ─────────────────────────────────────┘
```

---

```rs
struct Container<'a> {
    x: &'a i32,
    y: &'a i32,
}
```

---

```rs
fn main() {
    let x = 1;
    let v;
    {
        let y = 2;
        let f = Container { x: &x, y: &y };
        v = f.x;
    }
    println!("{}", *v);
}
```

---

![borrow checker](/static/slides/rust-introduction/Screenshot_20230221_184708.png)

---

```rs
struct Container<'a, 'b> {
    x: &'a i32,
    y: &'b i32,
}
```

---

Day 21:
```rs
fn chain_eval<'data, 'a>(
    start: &'data str,
    dependees: &'a HashMap<&'data str, Vec<&'data str>>,
    awaiting: &'a mut HashMap<&'data str, Shout>,
    finished: &'a mut HashMap<&'data str, i64>,
) {
    // -- snip --
}
```

---

```rs
pub fn run() -> () {
    // 'data lifetime start
    let data = utils::read_lines(utils::Source::Day(21)); 
    let data = parse_data(&data);

    println!("Day 21");
    println!("Part 1: {}", solve1(&data));
    println!("Part 2: {}", solve2(&data));
}
```

---

```rs [5-8|10-13]
fn test(arr: &[i32]) -> impl Iterator<Item=&i32> {
    return  arr.iter().cycle()
}

fn main() { // doesn't work
    let x = test(&vec![1, 2, 3]);
    x.take(9).for_each(|i| print!("{i}") );
}

fn main() { // works
    let x = test(&[1, 2, 3]);
    x.take(9).for_each(|i| print!("{i}") );
}
```

---

```rs [5-10]
fn test(arr: &[i32]) -> impl Iterator<Item=&i32> {
    return  arr.iter().cycle()
}

fn main() {
    let x = {
        let x: &'static [i32] = &[1, 2, 3];
        test(&x)
    };
    x.take(9).for_each(|i| print!("{i}") );
}
```

-----

### Unsafe

---

```rs
unsafe fn test(n: i32) -> i32 {
    // do something unsafe
    n * 5
}

fn test2() -> i32 {
    unsafe { test(4) }
}

fn main() {
    let x = unsafe { test(4) };
    let x = test2();
}
```

-----

### Traits

Haskell:
```hs
class Cool a where
    giveMeNumber :: a -> Integer
```

Rust:
```rs
trait Cool {
    fn give_me_number(&self) -> i32;
}
```

---

Haskell:
```hs
newtype Hello = Hello Integer

instance Cool Hello where
  giveMeNumber (Hello n) = n * 10
```

Rust:
```rs
struct Hello(i32);

impl Cool for Hello {
    fn give_me_number(&self) -> i32 { self.0 * 10 }
}
```

---

Haskell:
```hs
main = do
  let test = Hello 2
  putStrLn $ show $ giveMeNumber test
```

Rust:
```rs
fn main() {
    let test = Hello(2);
    println!("{}", test.give_me_number())
}
```

---

#### Some built in traits:
- ToString
- Display and Debug
- Default
- From and Into
- Clone and Copy
- Deref
- Iterator and IntoIterator

---

#### Default

```rs
#[derive(Default, Debug)]
struct Hello { a: i32, b: i32, c: Vec<i32> }

fn main() {
    let x = Hello { a: 22, ..Default::default() };
    println!("{x:?}")
}
```

```txt
Hello { a: 22, b: 0, c: [] }
```

---

#### Sized
```rs
// core::marker::Sized
pub trait Sized { }
```

```rs
fn test() -> dyn Iterator<Item=i32> {
    [1, 2, 3].into_iter()
}
```

![compiler](/static/slides/rust-introduction/Screenshot_20230223_191210.png)

---

```rs
fn test() -> Box<dyn Iterator<Item=i32>> + Sized {
    Box::from([1, 2, 3].into_iter())
}
```

---

#### Send & Sync
- A type is Send if it is safe to send it to another thread.
- A type is Sync if it is safe to share between threads (T is Sync if and only if &T is Send).

-----

### ?

```rs
fn test() -> Option<i32> {
    let mut test: Vec<i32> = vec![1, 2, 3];
    let x = test.pop()?;
    let y = test.pop()?.checked_add(22)?;
    Some(x * y)
}

fn main() {
    println!("{}", test().unwrap())
}
```
