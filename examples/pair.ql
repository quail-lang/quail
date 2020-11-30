def fst = fun p =>
    match p
        with pair a b => a

def snd = fun p =>
    match p
        with pair a b => b

def main =
    let p = pair zero (succ zero) in
    let x = println (fst p) in
    println (snd p)
