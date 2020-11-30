def one = Succ Zero
def two = Succ one
def three = Succ two
def four = add two two
def five = Succ four

def one_hundred = mul four (mul five five)

def sub = fun n m =>
    match m
        with Zero => n
        with Succ m' => (
            match n
                with Zero => Zero
                with Succ n' => sub n' m'
        )

def rem  = fun n m =>
    match less_than n m
        with True => n
        with False => rem (sub n m) m

def is_factor_of = fun n m => eq (rem m n) Zero

def count_factors_iter = fun n k acc =>
    match k
        with Zero => acc
        with Succ k' => (
            match is_factor_of k n
                with True => count_factors_iter n k' (Succ acc)
                with False => count_factors_iter n k' acc
        )

def count_factors = fun n => count_factors_iter n n Zero

def is_prime = fun n => eq (count_factors n) two

def eq = fun n m =>
    match n
        with Zero => (
            match m
                with Zero => True
                with Succ x => False
        )
        with Succ n' => (
            match m
                with Zero => False
                with Succ m' => eq n' m'
        )

def less_than = fun n m =>
    less_than_eq (Succ n) m

def less_than_eq = fun n m =>
    match n
        with Zero => True
        with Succ n' => (
            match m
                with Zero => False
                with Succ m' => less_than_eq n' m'
        )

def add = fun n m =>
    match n
        with Zero => m
        with Succ n => Succ (add n m)

def mul = fun n m =>
    match n
        with Zero => Zero
        with Succ n => add m (mul n m)


def pow = fun n m =>
    match m
        with Zero => Succ Zero
        with Succ m' => mul n (pow n m')

def sixteen = pow two (pow two two)

def repeat =
    fun f n =>
        repeat_iter f n n

def repeat_iter =
    fun f n k =>
        match k
            with Zero => Zero
            with Succ k' =>
                let x = f (sub n k)
                in (repeat_iter f n k')

def id = fun x => x

def print_prime = fun n =>
    match is_prime n
        with False => Zero
        with True => println n

def main = repeat print_prime sixteen
