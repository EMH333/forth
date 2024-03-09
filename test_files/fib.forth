\ If n1 is greater than 1, calculate n1-1 and n1-2, recurse twice,
\ add the results and return the sum. Otherwise, return n1 unchanged.
\ : 1- -1 + ;
\ original: : fib dup 1 > if 1- dup 1- recurse swap recurse + then ;
\ : fib dup 1 > if -1 + dup -1 + fib swap fib + then ;
: over swap dup rot rot ;
: fib 0 1 2 2 DO SWAP OVER + SWAP . LOOP DROP ;
6 fib .