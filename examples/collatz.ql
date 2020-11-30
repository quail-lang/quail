import nat

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

# For instance, starting with n = 12, one gets the sequence 12, 6, 3, 10, 5, 16, 8, 4, 2, 1.
def main : Top =
    let n = add ten two
    in let x = println (show n)

    in let n' = collatz n
    in let x' = println (show n')

    in let n'' = collatz n'
    in let x'' = println (show n'')

    in let n''' = collatz n''
    in let x''' = println (show n''')

    in let n'''' = collatz n'''
    in println (show n'''')
