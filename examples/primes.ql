# A program which prints out all of the prime numbers less than 16.

import nat

def one_hundred : Nat = mul four (mul five five)

def is_factor_of n m : Nat -> Nat -> Bool = eq (rem m n) zero

def count_factors_iter n k acc : Nat -> Nat -> Nat -> Nat =
    match k
        with zero => acc
        with succ k' => (
            match is_factor_of k n
                with true => count_factors_iter n k' (succ acc)
                with false => count_factors_iter n k' acc
        )

## Counts the numer of factors for a given natural number.
def count_factors n : Nat -> Nat = count_factors_iter n n zero

def is_prime n : Nat -> Bool = eq (count_factors n) two

def sixteen : Nat = pow two (pow two two)

def ap : (Nat -> Nat) -> Nat = fun f => f zero

def repeat_iter f n k : (Nat -> Unit) -> Nat -> Nat -> Unit =
    match k
        with zero => unit
        with succ k' =>
            let x = f (sub n k)
            in (repeat_iter f n k')


def repeat : (Nat -> Unit) -> Nat -> Unit = fun f n =>
    repeat_iter f n n

def print_prime n : Nat -> Unit =
    match is_prime n
        with false => unit
        with true => println (show n)

def main : Unit = repeat print_prime sixteen
