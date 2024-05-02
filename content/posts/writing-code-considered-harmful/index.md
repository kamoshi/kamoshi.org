---
title: Writing code considered harmful
date: 2023-06-14T22:09:06.127Z
---

When I initially delved into programming, I held the belief that it revolved solely around composing sets of instructions for the computer to execute. At a glance, it indeed appeared as such - merely commanding the computer to carry out tasks in a predetermined sequence, with the anticipation of obtaining a desirable outcome. This perception was further reinforced when observing how computers interpret code in assembly language, where each step is executed in a specific order within the CPU.

However, my view gradually changed as I was studying computer science at an university. First it was the object-oriented programming, which showed me the importance of choosing the right high level abstractions for a given problem. It started to seem to me as if programming was more about composing such abstractions to make problems easier to solve and easier to modify as needs change :cite[gamma1994design].

I also once thought that mathematics are not that important when it comes to computer science. I remember that at the start I had linear algebra and calculus courses, but I didn't really enjoy them much. Back then I didn't really understand the value of mathematics in this context, I just thought that they were things I had to pass on the way to „real” programming. To my horror it never seemed to end! It just became more subtly embedded within what I was doing all around.

For instance, statistics, and linear algebra, turned out to be the key to understanding machine learning. Without sufficient knowledge in that area of mathematics you can't really grasp how machine learning works on the fundamental level :cite[chollet2021deep]. It was at that point that I regretted not having worked harder at my linear algebra foundations.

A highlight of my studies was when I had a functional programming course, which felt like a U-turn probably on everything I learned up until that point. Suddenly, mutability was a bad thing, and everything was a function. I had to give up living in the object-oriented world. At that point I realized that there is in fact a different way to write software, and it's one deeply connected with nothing other than - mathematics. Okay.

Functional programming has the additional advantage over imperative and object-oriented paradigms that its abstractions are founded in mathematics. In fact, you can even write your programs using denotational demantics, which essentially means that code is more or less equivalent to mathematical equations. You can prove the program is correct by proving that the mathematical equivalent of it is :cite[10.1145/360303.360308].

It's really powerful.

## Summary

It was a pretty fun and interesting adventure, difficult at times, but worth it over all. In hindsight, I regret not paying more attention to mathematics right from the start. If I were to start over I would probably focus more on that and the theory of computation. If you're only just starting out then please remember, writing code is not computer science :cat:
