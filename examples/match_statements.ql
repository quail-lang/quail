def main = match Succ (Succ Zero)
    with Zero => println 123
    with Succ n => println n
