(define (loop-break)
  (display "testing loop with break\n  ")
  (define x 10)
  (loop
    (if (<= x 0) (break))
    (display x ", ")
    (set! x (- x 1))
  )
  (display "launch!\n")
)

(define (loop-break-continue)
  (display "testing loop with break and continue\n  ")
  (define x 10)
  (loop
    (cond ((<= x 0) (break))
          ((= 0 (% x 2))
            (begin
              (set! x (- x 1))
              (continue)
            ))
          (else (pass)))
    (display x ", ")
    (set! x (- x 1))
  )
  (display "launch!\n")
)

(define (application-start)
  (loop-break)
  (loop-break-continue)
)
