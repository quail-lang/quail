def one = succ zero
def two = succ one
def three = succ two
def four = add two two
def five = succ four
def ten = mul two five

def sub = fun n m =>
    match m
        with zero => n
        with succ m' => (
            match n
                with zero => zero
                with succ n' => sub n' m'
        )

def mul = fun n m =>
    match n
        with zero => zero
        with succ n => add m (mul n m)


def tail = fun l =>
    match l
        with nil => ?{This should be impossible}
        with cons h t => t

def up_to_iter = fun n k =>
    match k
        with zero => nil
        with succ k' =>
            let r = sub n k
            in cons r (up_to_iter n k')

def zip = fun xs ys =>
    match xs
        with nil => nil
        with cons x xs' => (
            match ys
                with nil => nil
                with cons y ys' =>
                    let head = pair x y in
                    let tail = zip xs' ys' in
                    cons head tail
        )

def zip_with = fun f xs ys =>
    let pairs = zip xs ys in
    map (fun p => f (fst p) (snd p))

def up_to = fun n => up_to_iter n n

def map = fun f xs =>
    match xs
        with nil => nil
        with cons x xs' => cons (f x) (map f xs')

def fst = fun p =>
    match p
        with pair a b => a

def snd = fun p =>
    match p
        with pair a b => b

def add = fun n m =>
    match n
        with zero => m
        with succ n => succ (add n m)

def fib_iter = fun n a b =>
    match n
        with zero => add a b
        with succ n' => fib_iter n' b (add a b)

def fib = fun n =>
    match n
        with zero => zero
        with succ n' => (
            match n'
                with zero => succ zero
                with succ n'' => fib_iter n'' zero (succ zero)
            )

def take = fun n xs =>
    match n
        with zero => nil
        with succ n' => (
            match xs
                with nil => nil
                with cons x xs' => cons x (take n' xs)
        )

def main =
    map (fun n => println (fib n)) (up_to ten)
