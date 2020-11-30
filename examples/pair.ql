def one = succ zero
def two = succ one
def three = succ two
def four = succ three
def five = succ four

def fst = fun p =>
    match p
        with pair a b => a

def snd = fun p =>
    match p
        with pair a b => b

def main =
    let p = pair four two in
    let x = println (fst p) in
    println (snd p)

