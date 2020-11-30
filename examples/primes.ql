# A program which prints out all of the prime numbers less than 16.

def one = succ zero
def two = succ one
def three = succ two
def four = add two two
def five = succ four

def one_hundred = mul four (mul five five)

def sub = fun n m =>
    match m
        with zero => n
        with succ m' => (
            match n
                with zero => zero
                with succ n' => sub n' m'
        )

def rem  = fun n m =>
    match less_than n m
        with true => n
        with false => rem (sub n m) m

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

def eq = fun n m =>
    match n
        with zero => (
            match m
                with zero => true
                with succ x => false
        )
        with succ n' => (
            match m
                with zero => false
                with succ m' => eq n' m'
        )

def less_than = fun n m =>
    less_than_eq (succ n) m

def less_than_eq = fun n m =>
    match n
        with zero => true
        with succ n' => (
            match m
                with zero => false
                with succ m' => less_than_eq n' m'
        )

def add = fun n m =>
    match n
        with zero => m
        with succ n => succ (add n m)

def mul = fun n m =>
    match n
        with zero => zero
        with succ n => add m (mul n m)


def pow = fun n m =>
    match m
        with zero => succ zero
        with succ m' => mul n (pow n m')

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

def id = fun x => x

def print_prime = fun n =>
    match is_prime n
        with false => zero
        with true => println n

def main = repeat print_prime sixteen
