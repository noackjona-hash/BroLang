fn fib(n)
    if n <= 1
        return n
    end
    set a to fib(n - 1)
    set b to fib(n - 2)
    return a + b
end

print fib(6)
