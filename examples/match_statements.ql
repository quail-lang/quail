def main = match succ (succ zero)
    with zero => println 123
    with succ n => println n
