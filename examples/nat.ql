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

def main : Top = println (show (add two three))
