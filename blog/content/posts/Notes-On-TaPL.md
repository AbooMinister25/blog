---
title = "Notes on Types and Programming Languages"
tags = ["programming", "type theory", "notes"]
date = "2024-11-30T4:00:00"
---

{{! note !}}  
**This is a live document, and will be updated as I read through the book.**
{{! end !}}


I've recently taken to reading [Types and Programming Languages](https://www.cis.upenn.edu/~bcpierce/tapl/) by Benjamin C. Pierce in an attempt to introduce myself to the basics of type theory and the like. The book in question is, from what I've found, considered an apt introduction to the aforementioned concepts, and seems to read well from my brief skimming of it thus far. I'll be using this document to consolidate my notes on the content as I read. 

The book mentions a mature understanding of mathematics, mainly discrete math, algorithms, and logic, namely as a product of rigorous undergraduate coursework, and familiarity with at some higher-order function programming language as a precursory requirement: none of which I satisfactorily meet. I suppose I'll have to see how much of this will fly over my head.

## Chapter 1: Introduction

This chapter defines what a "type system" is, and provides some relevant background.

- The book starts by offering a plausible definition of a "type system" as follows:
  
  - "A type system is a tractable syntactic method for proving the absence of certain program
    behaviors by classifying phrases according to the kinds of values they compute."

- However it is then stated that this definition is restricted to type systems in the context of being a tool for reasoning about programs.

- More generally, "the term type systems (or type theory) refers to a much broader field of study in logic, mathematics, and philosophy".

- This broader field predates programming, and were formalized in the early 1900s to avoid logical paradoxes.

- Types are now used in logic and proofs.

- There are two major branches to the study of type systems in computer science.
  
  - The practical branch, concerning the application of type systems to programming languages.
  
  - The abstract branch, connecting different lambda calculi and varieties of logic through something called the Curry-Howard correspondence.

- Type systems are conservative, they can prove that a given program won't have bad runtime behavior, but they also can reject programs that are otherwise valid at runtime.

- Programmers that work with richly typed languages can often expect that a program tends to work once it passes the typechecker.

- Both trivial mental slips and deeper conceptual errors can often be caught at the type level.

- This can vary depending on the expressiveness of the type system.

- Type checkers can also be important maintenance tools; changing a definition will raise type errors in all areas that refer to this definition and need to be updated accordingly.

- Type systems also promote abstraction, and prove useful in documentation and readability.

- Implementing a type system for a language which wasn't designed with type checking in mind is difficult.
  
  - Languages that don't have type systems have features or idioms that make typechecking difficult.

## Chapter 3: Untyped Arithmetic Expressions

The book establishes the following grammar for a very basic language of numbers and booleans.

```bnf
t ::=                           terms:
    true                        constant true
    false                       constant false
    if t then t else t          conditional
    0                           constant 0
    succ t                      sucessor
    pred t                      predecessor
    iszero t                    zero test
```


