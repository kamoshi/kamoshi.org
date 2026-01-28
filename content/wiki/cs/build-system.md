---
title: Build System
---

| Rebuilding Strategy \ Scheduler | Topological | Restarting | Suspending      |
| :------------------------------ | :---------- | :--------- | :-------------- |
| **Dirty Bit**                   | Make        | Excel      |                 |
| **Verifying Traces**            | Ninja       |            | Shake           |
| **Constructive Traces**         | Buck        |            | *Cloud Shake*   |
| **Deep Constructive Traces**    | CloudBuild  | Bazel      | **Nix**         |

## What to read

* https://nikhilism.com/post/2020/type-classes-build-systems/
* https://simon.peytonjones.org/build-systems-a-la-carte-theory-and-practice/
* https://edolstra.github.io/pubs/phd-thesis.pdf
