def one : Nat = succ zero
def two : Nat = succ one
def three : Nat = succ two
def four : Nat = succ three
def five : Nat = succ four
def six : Nat = succ five
def seven : Nat = succ six
def eight : Nat = succ seven
def nine : Nat = succ eight
def ten : Nat = succ nine

def add : Nat -> Nat -> Nat = fun n m =>
    match n
        with zero => m
        with succ n => succ (add n m)

def mul : Nat -> Nat -> Nat = fun n m =>
    match n
        with zero => zero
        with succ n => add m (mul n m)

def pow : Nat -> Nat -> Nat = fun n m =>
    match m
        with zero => succ zero
        with succ m' => mul n (pow n m')

def sub : Nat -> Nat -> Nat = fun n m =>
    match m
        with zero => n
        with succ m' => (
            match n
                with zero => zero
                with succ n' => sub n' m'
        )

def is_zero : Nat -> Bool = fun n =>
    match n
        with zero => true
        with succ x => false

def less_than_eq : Nat -> Nat -> Bool = fun n m =>
    match n
        with zero => true
        with succ n' => (
            match m
                with zero => false
                with succ m' => less_than_eq n' m'
        )

def less_than : Nat -> Nat -> Bool = fun n m =>
    less_than_eq (succ n) m

def rem : Nat -> Nat -> Nat = fun n m =>
    match less_than n m
        with true => n
        with false => rem (sub n m) m

def eq : Nat -> Nat -> Bool = fun n m =>
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

def one_hundred : Nat = mul four (mul five five)
def one_hundred_one : Nat = succ one_hundred

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

def main : Nat = is_prime (add ten nine)
