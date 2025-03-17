---
title: Flox
date: 2024-08-25T09:59:41Z
link: https://github.com/kamoshi/loxy
tech: [rust]
desc: >
    Small functional language written in Rust and compiled to WebAssembly
---

## Grammar

```
Program      ::= Sequence "EOF"
Sequence     ::= Expression? (";" Expression)*

Expression   ::= Data
               | Match
               | Let
               | If
               | While
               | Return
               | Block
               | Assignment

Data         ::= "data" IDENTIFIER ("|" IDENTIFIER (IDENTIFIER)*)*
Match        ::= "match" Expression ( "|" Expression IDENTIFIER* "->" Expression )*
Let          ::= "let" IDENTIFIER (IDENTIFIER)* "=" Expression
If           ::= "if" Expression Expression ("else" Expression)?
While        ::= "while" Expression Expression
Return       ::= "return" Expression?
Block        ::= "{" Sequence "}"
Assignment   ::= Call ("=" Assignment)?
Call         ::= Index (Index)*
Index        ::= Primary ("." IDENTIFIER)?

Primary      ::= IDENTIFIER
               | NUMBER
               | STRING
               | "true"
               | "false"
               | "nil"
               | Lambda
               | Parenthesized
               | Array

Lambda       ::= "fn" IDENTIFIER+ "->" Expression
Parenthesized ::= "(" (Expression ("," Expression)*)? ")"
Array        ::= "[" (Expression ("," Expression)*)? "]"
Arguments    ::= "(" (Expression ("," Expression)*)? ")"
```
