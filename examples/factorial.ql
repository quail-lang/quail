def one = succ zero
def two = succ one
def three = succ two
def four = succ three
def five = succ four

def add n m =
    match n
        with zero => m
        with succ n => succ (add n m)

def mul n m =
    match n
        with zero => zero
        with succ n => add m (mul n m)

def fact n =
    match n
        with zero => one
        with succ n' => mul n (fact n')

def main = println (fact five)
