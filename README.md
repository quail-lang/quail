## the Quail Language

Quail is a programming inspired by Haskell, Idris, and Elm. It is a language which aims to explore
the potential of language design when no compromises are made against the fundamental notions of
purity and totality. Purity is the idea that evaluation of an expression never produces an
observable side-effect inside the runtime. Any variable may be substituted for its definition, and
the semantics of the program are preserved. Totality is the idea that all functions are well-defined
in a mathematical sense. That is, they must guarantee full case coverage, they not allowed to raise
exceptions, and they must provably not get stuck in loops.
