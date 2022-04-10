(define (application-start)
  (display "Please input a number: ")
  (define the-string (read-line))
  (define the-number (string->int the-string))
  (display the-number
           " + 1 is: "
           (+ the-number 1)
           "\n"
  )
  (cond ((or (= the-number 114514)
             (= the-number 1919810))
         (display "The number is so stench!"))
        (else (pass)))
)
