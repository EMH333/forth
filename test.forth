." Reset... " RESET
." Check comments... " \ Yes, we support the new-style comments :-)
." Computing simple addition of 3 + 4... " 3 4 + .
." Is 1 = 2 ?... " 1 2 = .
." Is 1 > 2 ?... " 1 2 > .
." Is 1 < 2 ?... " 1 2 < .
." Define pi at double-word precision... " : pi 355 113 */ ;
." Use definition to compute 10K times PI... " 10000 pi .
." Check: 23 mod 7... " 23 7 MOD .
." Defining 1st level function1... " : x2 2 * ;
." Defining 1st level function2... " : p4 4 + ;
." 2nd level word using both - must print 24... " 10 x2 p4 .
." Defining a variable with value 123... " 123 variable ot3
." Printing variable's value... " ot3 @ .
." Defining The Constant (TM)... " 42 constant lifeUniverse
." Printing The Constant (TM)... " lifeUniverse .
." Setting the variable to The Constant (TM)... " lifeUniverse ot3 !
." Printing variable's value... " ot3 @ .
." Setting the variable to hex 0x11... " $11 ot3 !
." Printing variable's value... " ot3 @ .
." Defining helper... " : p5 5 U.R . ;
." Defining 3 times loop... " : x3lp 3 0 DO I p5 LOOP ;
." Calling loop... " x3lp
." Defining loop calling loop 2 times... " : x6lp 2 0 DO x3lp LOOP ;
." Nested-looping 2x3 times... " x6lp
." Inline: " : m 3 1 DO 3 1 DO CR J p5 I p5 ." = " J I * p5 LOOP LOOP ;
." Use inline loops with two indexes... " m
." Make multiples of 7 via DUP... " : m7s 10 0 DO DUP I * . LOOP DROP ;
." Print them and DROP the 7... " 7 m7s
." Reset... " RESET
\ Time for Turing completeness...
." Let's do Fizz-Buzz! " \ Turing Completeness check...
\ fizz ( n -- 0_or_1 n )
." Define fizz... " : fizz DUP 3 MOD 0 = IF ." fizz " 1 ELSE 0 THEN SWAP ;
\ buzz ( n -- 0_or_1 n )
." Define buzz... " : buzz DUP 5 MOD 0 = IF ." buzz " 1 ELSE 0 THEN SWAP ;
\ emitNum ( 0_or_1 0_or_1 n -- )
." Define emitNum... " : emitNum ROT ROT + 0 = if . ELSE DROP THEN ;
\ mainloop ( n -- )
." Define mainloop... " : mainloop fizz buzz emitNum ;
\ fb ( -- )
." Define fizzbuzz... " : fb 37 1 DO I mainloop LOOP ;
." Run it! " fb
." Nested Ifs 1 " 1 1 IF ." First " IF ." Second " THEN ELSE ." Else " IF ." True In Else " THEN THEN
." Nested Ifs 0 " 0 0 IF ." A " IF ." B " THEN ELSE ." C " IF ." True In Else " ELSE ." E " THEN THEN
." Report memory usage... " .S