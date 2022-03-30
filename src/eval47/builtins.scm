(defun (unused x) x)

(defun (unimplemented) (raise "unimplemented"))

(defun (--pr47-builtin-cons fst snd) (vector fst snd))

(defun (cons x y) (--pr47-builtin-cons x y))

(defun (car x) (vector-ref x 0))
(defun (cdr x) (vector-ref x 1))
(defun (set-car! x value) (vector-set! x 0 value))
(defun (set-cdr! x value) (vector-set! x 1 value))

(defun (list-length list)
  (defun (list-length-impl list n)
    (cond ((= nil list) n)
          (else (list-length-impl (cdr list) (+ n 1)))))
  (list-length-impl list 0))

(defun (list-ref list n)
  (defun (list-ref-impl list n)
    (cond ((= n 0) (car list))
          (else (list-ref-impl (cdr list) (- n 1)))))
  (cond ((>= n (list-length list)) (raise "IndexOutOfBounds"))
        (else (list-ref-impl list n))))

(defun (map f list)
  (cond ((= nil list) nil)
        (else (cons (f (car list)) (map f (cdr list))))))

(defun (display-list list)
  (defun (display-list-impl list)
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
