(define (stdlib-version) (vector 0 0 1))

(define (unused x) x)

(define (unimplemented) (raise "unimplemented"))

(define (--pr47-builtin-cons fst snd) (vector fst snd))

(define (cons x y) (--pr47-builtin-cons x y))

(define (car x) (vector-ref x 0))
(define (cdr x) (vector-ref x 1))
(define (set-car! x value) (vector-set! x 0 value))
(define (set-cdr! x value) (vector-set! x 1 value))

(define (list-length list)
  (define (list-length-impl list n)
    (cond ((= nil list) n)
          (else (list-length-impl (cdr list) (+ n 1)))))
  (list-length-impl list 0))

(define (list-ref list n)
  (define (list-ref-impl list n)
    (cond ((= n 0) (car list))
          (else (list-ref-impl (cdr list) (- n 1)))))
  (cond ((>= n (list-length list)) (raise "IndexOutOfBounds"))
        (else (list-ref-impl list n))))

(define (map f list)
  (cond ((= nil list) nil)
        (else (cons (f (car list)) (map f (cdr list))))))

(define (display-list list)
  (define (display-list-impl list)
    (cond ((= nil (cdr list)) (display (car list)))
          (else
            (begin
              (display (car list) ", ")
              (display-list-impl (cdr list))
            )
          )
    )
  )
  (cond ((= nil list) (display "nil"))
        (else (display-list-impl list))
  )
)
