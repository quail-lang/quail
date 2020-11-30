# A program which prints out all of the prime numbers less than 16.

import nat

def one_hundred = mul four (mul five five)

def is_factor_of = fun n m => eq (rem m n) zero

def count_factors_iter = fun n k acc =>
    match k
        with zero => acc
        with succ k' => (
            match is_factor_of k n
                with true => count_factors_iter n k' (succ acc)
                with false => count_factors_iter n k' acc
        )

# Counts the numer of factors for a given natural number.
def count_factors = fun n => count_factors_iter n n zero

def is_prime = fun n => eq (count_factors n) two

def sixteen = pow two (pow two two)

def repeat =
    fun f n =>
        repeat_iter f n n

def repeat_iter =
    fun f n k =>
        match k
            with zero => zero
            with succ k' =>
                let x = f (sub n k)
                in (repeat_iter f n k')

def print_prime = fun n =>
    match is_prime n
        with false => zero
        with true => println n

def main = repeat print_prime sixteen
