---
title: Haskell FFI
date: 2025-07-29T18:30:29.523Z
tags: [haskell, rust, c, ffi]
desc: >
  Integrating different programming languages can be a complex endeavor. This
  article explores the fascinating world of Foreign Function Interface (FFI),
  demonstrating how to bridge Haskell and Rust.
---

Integrating different programming languages can be a complex but incredibly
rewarding endeavor. For me, it's been a fascinating journey into the world of
Foreign Function Interface (FFI), specifically exploring how to make Haskell and
Rust talk to each other. It's been a mix of head-scratching moments and amazing
breakthroughs, truly a lot of fun.

##  My first steps: basic arithmetic with Rust

My adventure began with a simple goal: could I get Haskell to call basic
arithmetic functions written in Rust? I started by creating a small Rust
library. I defined `add` and `mul` functions, making sure they were
`#[no_mangle]` and `extern "C"` so Haskell could find them easily.

```rust
#[unsafe(no_mangle)]
pub extern "C" fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[unsafe(no_mangle)]
pub extern "C" fn mul(left: u64, right: u64) -> u64 {
    left * right
}
```

On the Haskell side, I used the `CApiFFI` extension and `Foreign.C.Types` to
declare these functions. It's just as if I was writing a contract between the
two languages, which for some reason had to be in C.

```haskell
{-# LANGUAGE CApiFFI #-}
import Foreign.C.Types

foreign import ccall "add" add :: CInt -> CInt -> IO CInt
foreign import ccall "mul" mul :: CInt -> CInt -> IO CInt

main :: IO ()
main = do
  print 33
```

Loading this into GHCi, Haskell's interactive environment, with the Rust library
(`ghci -L./rusty/lib/ -lrusty ./Main.hs`), was my first big win. Seeing `mul 55
55` return `3025` right there in the prompt felt like magic! This initial
success confirmed my basic FFI setup was working.

```
ghci -L./rusty/lib/ -lrusty ./Main.hs
Ok, one module loaded.
ghci> mul 55 55
3025
```

## Passing complex data structures and callbacks

My next challenge was more intricate: could I create a Rust object from Haskell,
manipulate it, and then "finish" its configuration? This meant dealing with more
complex data structures and managing memory across the language boundary.

### Rust side: opaque pointers to the rescue

To pass complex Rust types to Haskell without revealing their internal
structure, I learned about opaque pointers. I defined a `WebsiteConfigOpaque`
struct and wrote functions (`new_website` and `finish`) to allocate and
deallocate the Rust `WebsiteConfiguration` on the heap, returning a raw pointer
to Haskell. This felt a bit like passing a sealed box across the border. Haskell
knew it was a box, but not what was inside.

```rust
use hauchiwa::WebsiteConfiguration;

#[repr(C)]
pub struct WebsiteConfigOpaque {
  _private: [u8; 0],
}

type WebsiteConfigHandle = *mut WebsiteConfigOpaque;

#[unsafe(no_mangle)]
extern "C" fn new_website() -> WebsiteConfigHandle {
  let config: WebsiteConfiguration<()> = hauchiwa::Website::configure();
  let config = Box::new(config);
  Box::into_raw(config) as WebsiteConfigHandle
}

#[unsafe(no_mangle)]
pub extern "C" fn finish(ptr: WebsiteConfigHandle) -> i32 {
  if ptr.is_null() {
      return -1; // Null pointer error
  }
  let config: Box<hauchiwa::WebsiteConfiguration<()>> =
      unsafe { Box::from_raw(ptr as *mut hauchiwa::WebsiteConfiguration<()>) };
  let config = *config;
  let mut website = hauchiwa::WebsiteConfiguration::finish(config);
  match website.build(()) {
      Ok(_) => 1,
      Err(_) => 0,
  }
}
```

### Haskell side: interacting with opaque pointers

On the Haskell side, I mirrored the opaque type and defined foreign imports for
`new_website` and `finish`. The `IO` monad was my trusty companion here,
handling the side effects of memory operations.

```haskell
module Main where
import Foreign (Int32, Ptr)
import Foreign.C.Types

data WebsiteConfigOpaque
type WebsiteConfigHandle = Ptr WebsiteConfigOpaque

foreign import ccall "new_website" newWebsite :: IO WebsiteConfigHandle
foreign import ccall "finish" finish :: WebsiteConfigHandle -> IO Int32

main :: IO ()
main = do
website <- newWebsite
res <- finish website
print res
```

Running this in GHCi worked perfectly, returning `0`, which meant the website
configuration process completed without a hitch. Another small victory!

```
ghci -L./lib -lrusty ./Main.hs
Ok, one module loaded.
ghci> main
Loaded git repository data (+2ms)
Running Hauchiwa in build mode.
Cleaned the dist directory (+0ms)
0
```

The result of `0` indicates no errors during the website configuration process.

## The final frontier: Haskell callbacks in Rust

This was the part that truly pushed my understanding - passing a Haskell
function to Rust, allowing Rust to call *back* into Haskell. This felt like the
ultimate challenge in FFI.

### The GHCi `wrapper` wall

I hit a wall when I tried to use the `wrapper` keyword in GHCi. When I added the
`foreign import ccall "wrapper"` declaration to my Haskell file and tried to
load it, GHCi threw an error:

```
GHC.Linker.Loader.dynLoadObjs: Loading temp shared object failed
During interactive linking, GHCi couldn't find the following symbol:
  librusty.so: cannot open shared object file: No such file or directory
```

It was frustrating because I knew the library was there, and compiling to a
standalone binary (`ghc -O2 Main.hs -L./lib -lmylib -o out;
LD_LIBRARY_PATH=./lib ./out`) worked perfectly. I was so close, but GHCi was
being stubborn. I wondered if it was a limitation of GHCi itself or if I was
missing something fundamental.

### Rust side: expecting a callback function pointer

Meanwhile, on the Rust side, I set up my `do_callback` function to accept a
`HaskellCallback` type, which is essentially a C-compatible function pointer.

```rust
pub type HaskellCallback = extern "C" fn(i32) -> i32;

#[unsafe(no_mangle)]
pub extern "C" fn do_callback(f: HaskellCallback) -> i32 {
    println!("I run inside Rust");
    f(33)
}
```

### Haskell side: creating and passing a function pointer

In Haskell, I used the `wrapper` keyword to create a C-callable function pointer
from my `haskellCallback` function. This `FunPtr` would then be passed to the
Rust `do_callback`.

```haskell
module Main where
import Foreign (Int32, Ptr)
import Foreign.C.Types
import Foreign.Ptr (FunPtr)

-- The Haskell function to be passed as a callback
haskellCallback :: CInt -> IO CInt
haskellCallback x = do
  putStrLn $ "Haskell received: " ++ show x
  return (x + 99)

-- Turn Haskell function into a C-callable function pointer
foreign import ccall "wrapper"
  mkCallback :: (CInt -> IO CInt) -> IO (FunPtr (CInt -> IO CInt))

-- Import the Rust function that accepts a callback
foreign import ccall "do_callback"
  doCallback :: FunPtr (CInt -> IO CInt) -> IO CInt

main :: IO ()
main = do
  cb <- mkCallback haskellCallback
  res2 <- doCallback cb
  print res2
```

I was stuck on the GHCi issue for a bit. I even asked around, and someone
mentioned that what I was doing was "pretty weird" and suggested using Cabal.
While I knew Cabal was the standard, I was determined to see what I could
accomplish with plain GHC and FFI. I suspected it was a linker issue, not a
fundamental limitation.

Then, after some more digging (and a helpful Stack Overflow link), I had a
breakthrough! I realized that in addition to the `-L` and `-l` flags for GHCi, I
also needed to set the `LD_LIBRARY_PATH` environment variable before launching
GHCi.

```
LD_LIBRARY_PATH=./lib ghci -L./lib -lrusty ./Main.hs
GHCi, version 9.6.7: https://www.haskell.org/ghc/  :? for help
[1 of 2] Compiling Main             ( Main.hs, interpreted )
Ok, one module loaded.
ghci> main
I run inside Rust
Haskell received: 33
132
ghci> :reload
[1 of 2] Compiling Main             ( Main.hs, interpreted ) [Source file changed]
Ok, one module loaded.
ghci> main
I run inside Rust
Haskell received: 33
9955
```

It worked! Seeing "I run inside Rust" followed by "Haskell received: 33" and
then the final `9955` (because I changed the return value of `haskellCallback`
during a reload) was incredibly satisfying. I could `:reload` my Haskell code in
GHCi, and the FFI connection to the Rust library remained intact. It turns out,
FFI in Haskell is quite simple for the user, it's just that some edge cases
aren't well-documented, making it easy to hit a wall.

## Conclusion

This entire exploration has been a blast. It showed me the immense power of
Foreign Function Interface, enabling Haskell and Rust to communicate seamlessly,
share data, and even pass functions back and forth. While there were moments of
frustration, especially with GHCi's specific linking requirements, overcoming
those hurdles was incredibly rewarding. It's amazing how much you can achieve
with FFI, even without using any more complex build systems like Cabal for these
specific interactions. This just goes to show how much effort went into
engineering GHC and GHCi, chapeau bas.
