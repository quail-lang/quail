import nat

def mul : Nat -> Nat -> Nat = fun n m =>
    match n
        with zero => zero
        with succ n => add m (mul n m)

def fact : Nat -> Nat = fun n =>
    match n
        with zero => one
        with succ n' => mul n (fact n')

def main : Top = println (show (fact five))
