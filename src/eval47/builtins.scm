(defun (--builtin-cons fst snd)
    (lambda (choice)
        (cond ((= choice 0) fst)
              ((= choice 1) snd)
              (else (dbg-int choice)))
    )
)

(defun (cons x y) (--builtin-cons x y))

(defun (car pair) (pair 0))
(defun (cdr pair) (pair 1))

(defun (fibonacci n)
    (cond ((= n 0) 0)
          ((= n 1) 1)
          (else (+ (fibonacci (- n 1))
                   (fibonacci (- n 2))))
    )
)

(defun (application-start)
  (dbg-int (fibonacci 35))
)
