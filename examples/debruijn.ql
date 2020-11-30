def const : Nat -> Nat -> Nat = fun n n => n$1

def main : Top = println (show (const zero (succ zero)))
