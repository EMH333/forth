\ Time for Turing completeness...
." Let's do Fizz-Buzz! " \ Turing Completeness check...
: fizz DUP 3 MOD 0 = IF ." fizz " 1 ELSE 0 THEN SWAP ;
: buzz DUP 5 MOD 0 = IF ." buzz " 1 ELSE 0 THEN SWAP ;
: emitNum ROT ROT + 0 = if . CR ELSE DROP CR THEN ;
: mainloop fizz buzz emitNum ;
: fb 99999999 1 DO I mainloop LOOP ;
." Run it! "
fb
