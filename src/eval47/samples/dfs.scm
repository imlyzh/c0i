(define (fold list init proc)
  (cond ((= nil list) init)
        (else (fold (cdr list)
                    (proc init (car list))
                    proc))
  )
)

(define (count-list list state)
  (fold list
        0
        (lambda (acc item)
          (cond ((= item state) (+ acc 1))
                (else acc))
        )
  )
)

(define (dfs summarise state-chain n-steps-left)
  (cond ((= n-steps-left 0)
          (let ((agree-count (count-list state-chain 1))
                (disagree-count (count-list state-chain -1))
                (neutral-count (count-list state-chain 0)))
            (begin
              (cond ((> agree-count disagree-count)
                      (object-set! summarise "pass" (+ 1 (object-get summarise "pass"))))
                    ((>= disagree-count (/ (list-length state-chain) 2))
                      (object-set! summarise "pass" (+ 1 (object-get summarise "pass"))))
                    (else
                      (object-set! summarise "fail" (+ 1 (object-get summarise "fail"))))
              )
            )
          )
        )
        (else
          (begin
            (dfs summarise (cons 1 state-chain) (- n-steps-left 1))
            (dfs summarise (cons 0 state-chain) (- n-steps-left 1))
            (dfs summarise (cons -1 state-chain) (- n-steps-left 1))
          )
        )
  )
)

(define (application-start)
  (define summarise (object))
  (object-set! summarise "pass" 0)
  (object-set! summarise "fail" 0)
  (map
    (lambda (x)
      (dfs summarise nil x)
      (display "x = "
               x
               ", pass count = "
               (object-get summarise "pass")
               ", fail count = "
               (object-get summarise "fail")
               "\n"
      ))
    '(1 2 3 4 5 6 7 8 9 10))
)
