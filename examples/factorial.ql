def add = fun n m =>
    match n
        with Zero => m
        with Succ n => Succ (add n m)

def one = Succ Zero
def two = Succ one
def three = Succ two
def four = Succ three
def five = Succ four

def mul = fun n m =>
    match n
        with Zero => Zero
        with Succ n => add m (mul n m)

def fact = fun n =>
    match n
        with Zero => one
        with Succ n' => mul n (fact n')

def main = println (fact five)
