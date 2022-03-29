(define (fibonacci n)
  (cond ((= n 0) 0)
        ((= n 1) 1)
        (else (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))

(define (dbg-int-fn x)
  (display x "\n"))

(define (application-start)
  (define the-list '(1 2 3 4 5 6 7 8 9 10))

  (display "the list is: ")
  (display-list the-list)
  (display "\n")

  (display "when printed in builtin display form: " the-list "\n")

  (display "the length of the list is "
           (list-length the-list)
           "\n")
  (display "the last element of the list is "
           (list-ref the-list (- (list-length the-list) 1))
           "\n")

  (display "calculating fibonacci: ")
  (display-list (map fibonacci the-list))
  (display "\n")
)
