import nat
import pair

def head = fun xs =>
    match xs
        with nil => ?{head of empty list}
        with cons x xs' => x

def tail = fun xs =>
    match xs
        with nil => ?{tail of empty list}
        with cons x xs' => xs'

def length = fun xs =>
    match xs
        with nil => zero
        with cons x xs' => succ (length xs')

def map = fun f xs =>
    match xs
        with nil => nil
        with cons x xs' => cons (f x) (map f xs')

def filter = fun p xs =>
    match xs
        with nil => nil
        with cons x xs' => (
            match p x
                with true => cons x (filter p xs')
                with false => filter p xs'
        )

def fold = fun z s xs =>
    match xs
        with nil => z
        with cons x xs' => s z (fold z s xs')

def up_to_iter = fun n k =>
    match k
        with zero => nil
        with succ k' =>
            let r = sub n k
            in cons r (up_to_iter n k')

def up_to = fun n => up_to_iter n n

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

def take = fun n xs =>
    match n
        with zero => nil
        with succ n' => (
            match xs
                with nil => nil
                with cons x xs' => cons x (take n' xs)
        )

def main =
    println (head (cons zero nil))
