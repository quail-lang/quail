[![CircleCI](https://circleci.com/gh/quail-lang/quail.svg?style=svg)](https://circleci.com/gh/quail-lang/quail)
[![Docs Status](https://docs.rs/quail/badge.svg)](https://docs.rs/quail)
[![Crates.io](https://img.shields.io/crates/v/quail.svg)](https://crates.io/crates/quail)

* [Introduction](#introduction)
* [Getting Started](#getting-started)
* [Basics](#basics)
* [Vim Highlighting](#vim-highlighting)

## Introduction

Quail is a programming inspired by Haskell, Idris, and Elm. It aims to explore
the potential of language design when no compromises are made against the
fundamental notions of purity and totality.

Purity is the idea that the evaluation of an expression never produces any
side-effect observable to the runtime. In Quail, all expressions represent
constant values. A variable, once defined, will never change its value, and so
may always be substituted for its definition. Purity demands that the author be
disciplined about the flow of data in his code. And dually, the maintainer is
freed from worrying about non-local behavior of a program.

Totality is the idea that evaluating any expression eventually results in a
value of the expected type. In Quail, match statements must cover all possible
cases. There is no notion of exceptions. Programs must not get stuck in
infinite loops. Recursion must always be well-founded. Totality allows Quail to
avoid the need for a preferred evaluation order (eg, strict vs lazy). It also
permits a meaningful distinction between inductive data (such as lists) and
coinductive data (such as streams).

Quail is meant to be a beginner-friendly language. Because of their academic
origins, functional programming languages have a reputation for being esoteric
and difficult to learn. This is unfortunate because the advantages of functional
programming are afforded by the basic ideas, such as strong typing, immutability,
pattern-matching, and the overall principled design of the lambda calculus. Quail
designed to be minimal, elegant, and above all else, easy to learn.

## Getting started

To get started, clone the repository, then run one of the example programs:

    $ git clone https://github.com/quail-lang/quail
    $ cd quail
    $ cargo run --release examples/primes.ql
       Compiling quail v0.1.0 (/home/quail-lang/projects/quail)
           Finished release [optimized] target(s) in 1.18s
                Running `target/release/quail examples/primes.ql`
                Loading "examples/primes.ql"
                Loading "examples/nat.ql"
                2
                3
                5
                7
                11
                13

## Basics

The most basic type in Quail is `Nat`, short for natural number. `Nat`s are constructed through the
constructors `zero` and `succ`. The number `zero` should be familiar to you. The function `succ` is
short for "successor". It means "one more than", and since it is a function, it must be applied to
another `Nat`. So the number one can be expressed as `succ zero`. And the number two can be
expressed as `succ (succ zero)`.

Every natural number can be expressed this way. There are no number literals, so you can't write
something like `3`. Instead, you have to construct the number you want explicitly: `succ (succ (succ
zero))`.

When you want to print a number to the screen, you can use the builtin `show` function, which turns
a `Nat` into a `Str`, and the `println` function, which prints a `Str` to the screen.

Here is a short program in Quail to get started with `Nat`s:

    # tutorial.ql
    def main : Top = println (show (succ (succ (succ zero))))

You can save this to the file `tutorial.ql` and then run it like this:

    $ quail tutorial.ql
    3

And you see the number `3` was printed to the console.

You can define a variable using the `def` keyword. So perhaps we want to define the first few
naturals so we don't have to type them over and over again:

    # tutorial.ql
    def one : Nat = succ zero
    def two : Nat = succ one
    def three : Nat = succ two

    def main : Top = println (show three)

Saving and running it again will show the same result.

Notice that `one`, `two`, and `three` all have `: Nat` written after them. The syntax `:` is
pronounced "has the type". So when we write `def three : Nat = ...`, we are defining a new variable
`three` which has the type `Nat`. In Quail, all top level definitions must be annotated with their
type.

To make decisions in Quail, we use `match` statements. A `match` statement will look at a value we
give it and then determine what action to take from there. For instance, if we want to see if a
number is zero or not, we can write this:

    # tutorial.ql
    def one : Nat = succ zero
    def two : Nat = succ one
    def three : Nat = succ two

    def main : Top = match three
        with zero => println "is zero"
        with succ n => println "is not zero"

Underneath the `match` statement, we have two lines that start with the keyword `with`. The `with`
keyword is always followed by a pattern, and the pattern is what we are trying to match against.
Lastly, we have a fat arrow `=>` followed by the expression we want in the case of a match.

The first with clause says, "when we match with `zero`, print `is zero` to the screen". The other
says "when we match with `succ`, print `is not zero` to the screen". The variable `n` after `succ`
is not used in this example, but it is there to tell us that the pattern match creates a new
variable `n` which would tell us what `Nat` you need to call `succ` on to get `three`, the value
we're matching against.

One funny thing about Quail is that while `zero` and `succ` are built into the language, the
familiar operation of addition is not. In order to perform addition, we must first define it.  The
way we do that is by using `match` togethr with a technique known as recursion:

    # tutorial.ql
    def one : Nat = succ zero
    def two : Nat = succ one
    def three : Nat = succ two

    def add : Nat -> Nat -> Nat =
         fun n m => match n
            with zero => m
            with succ n' => succ (add n' m)

    def main : Top = println (show (add two three))

Here, we have defined a new function `add`. You can see it has type `Nat -> Nat -> Nat`, meaning
that it takes two `Nat`s as arguments and returns a `Nat` as a result. It is defined as a function
using the `fun` keyword and we name its two arguments `n` and `m` respectively.

Once we have taken the two numbers as inputs, we proceed by matching on `n`. This allows us to break
`n` apart and look at the pieces. We can then think about how addition would work in either of the
two cases that make up `Nat`: `zero` and `succ`.

When `n` matches with `zero`, we think about what the value of `add zero m` should be. That seems
easy enough: adding `zero` to anything should just give us that thing unchanged. And so we have
`with zero => m` telling us exactly that.

The next line is a little more difficult. First, when we match `n` against the pattern `succ n'`, we
get a new variable to work with: `n'`. Because we are matching `n` with `succ n'`, these two
expressions are equal: `n = succ n'`. If you think about this for a moment, that means that `n'` is
the number one smaller than `n`.

Lastly, we make a recursive call to `add`. This means that `add` is going to be defined in terms of
itself. You might think that this creates a kind of circular logic. But as long as we are careful,
we can avoid any true circularity. We call `add` with the argument `n'` and `m`, and since `n'` is
always going to be smaller than `n`, the repeated calls to `add` will eventually bring `n` down to
`zero`, and our recursion will terminate.

You can see more examples of the `Nat` in [nat.ql](https://github.com/quail-lang/quail/blob/master/examples/nat.ql).

## Vim Highlighting

If you use vim, you can install the syntax highlighting like this:

    $ cp -r quail.vim/ ~/.vim/bundle/
