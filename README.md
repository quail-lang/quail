[![CircleCI](https://circleci.com/gh/quail-lang/quail.svg?style=svg)](https://circleci.com/gh/quail-lang/quail)

Quail is a programming inspired by Haskell, Idris, and Elm. It is a language which aims to explore
the potential of language design when no compromises are made against the fundamental notions of
purity and totality. Purity is the idea that evaluation of an expression never produces an
observable side-effect inside the runtime. Any variable may be substituted for its definition, and
the semantics of the program are preserved. Totality is the idea that all functions are well-defined
in a mathematical sense. That is, they must guarantee full case coverage, they not allowed to raise
exceptions, and they must provably not get stuck in loops.

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
