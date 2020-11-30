[![CircleCI](https://circleci.com/gh/quail-lang/quail.svg?style=svg)](https://circleci.com/gh/quail-lang/quail)

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

If you use vim, you can install the syntax highlighting like this:

    $ cp -r quail.vim/ ~/.vim/bundle/
