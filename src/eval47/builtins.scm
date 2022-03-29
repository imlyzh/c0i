(defun (--pr47-builtin-cons fst snd)
    (lambda (choice)
        (cond ((= choice 0) fst)
              ((= choice 1) snd)
              (else (raise "God damn it what the hell are you doing?")))
    )
)

(defun (require x) false)

(defun (cons x y) (--pr47-builtin-cons x y))

(defun (car pair) (pair 0))
(defun (cdr pair) (pair 1))

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
