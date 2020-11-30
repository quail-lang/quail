import nat
import list

def half : Nat -> Nat = fun n =>
    match n
        with zero => zero
        with succ n' => (
            match n'
                with zero => zero
                with succ n'' => succ (half n''))

def collatz : Nat -> Nat = fun n =>
    match rem n two
        with zero => half n
        with succ n' => add (mul three n) one

def collatz_sequence : Nat -> List = fun n =>
    match eq n one
        with true => cons one nil
        with false => let x = println (show n) in cons n (collatz_sequence (collatz n))

def main : Top =
    let xs = collatz_sequence five
    in println (show_list xs)
