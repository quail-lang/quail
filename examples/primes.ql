# A program which prints out all of the prime numbers less than 16.

import nat

def one_hundred : Nat = mul four (mul five five)

def is_factor_of : Nat -> Nat -> Bool = fun n m  =>
    eq (rem m n) zero

def count_factors_iter : Nat -> Nat -> Nat -> Nat = fun n k acc =>
    match k
        with zero => acc
        with succ k' => (
            match is_factor_of k n
                with true => count_factors_iter n k' (succ acc)
                with false => count_factors_iter n k' acc
        )

## Counts the numer of factors for a given natural number.
def count_factors : Nat -> Nat = fun n =>
    count_factors_iter n n zero

def is_prime : Nat -> Bool = fun n =>
     eq (count_factors n) two

def sixteen : Nat = pow two (pow two two)

def repeat_iter : (Nat -> Top) -> Nat -> Nat -> Top = fun f n k =>
    match k
        with zero => top
        with succ k' =>
            let x = f (sub n k)
            in (repeat_iter f n k')


def repeat : (Nat -> Top) -> Nat -> Top = fun f n =>
    repeat_iter f n n

def print_prime : Nat -> Top = fun n =>
    match is_prime n
        with false => top
        with true => println (show n)

def main : Top = repeat print_prime sixteen
