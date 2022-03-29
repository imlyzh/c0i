(defun (unused x) x)

(defun (unimplemented) (raise "unimplemented"))

(defun (--pr47-builtin-cons fst snd)
    (define vec (vector fst snd))
    (lambda (choice param)
        (cond ((= choice 0) (vector-ref vec 0))
              ((= choice 1) (vector-ref vec 1))
              ((= choice 2) (vector-set! vec 0 param))
              ((= choice 3) (vector-set! vec 1 param))
              (else (raise "God damn it what the hell are you doing?")))
    )
)

(defun (cons x y) (--pr47-builtin-cons x y))

(defun (car pair) (pair 0 nil))
(defun (cdr pair) (pair 1 nil))

(defun (set-car! pair value) (pair 2 value))
(defun (set-cdr! pair value) (pair 3 value))

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
