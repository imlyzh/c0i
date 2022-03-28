(define (fibonacci n)
    (cond ((= n 0) 0)
          ((= n 1) 1)
          (else (+ (fibonacci (- n 1))
                   (fibonacci (- n 2))))
    )
)

(define (list-ref list n)
    (cond ((= n 0) (car list))
          (else (list-ref (cdr list) (- n 1)))
    )
)

(define (test-bool bool-value)
    (cond ((= bool-value false) (dbg-str "false"))
          ((= bool-value true) (dbg-str "true"))
    )
)

(define (application-start)
    (list-ref '(1 2 3) 3)

    (test-bool (and false (print-str-and-ret "test0" true)))
    (test-bool (and true (print-str-and-ret "test1" true)))
    (test-bool (or true (print-str-and-ret "test2" false)))
    (test-bool (or false (print-str-and-ret "test3" true)))
)

(define (print-str-and-ret str val)
    (dbg-str str)
    val
)
