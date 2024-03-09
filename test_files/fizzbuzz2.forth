\ Time for Turing completeness...
." Let's do Fizz-Buzz! " \ Turing Completeness check...
: fizz I DUP 3 MOD 0 = IF ." fizz " 1 ELSE 0 THEN SWAP ;
: buzz 5 MOD 0 = IF ." buzz " 1+ THEN ;
: emitNum 0 = IF I . THEN CR ;
: mainloop fizz buzz emitNum ;
: fb 37 1 DO mainloop LOOP ;
." Run it! "
fb