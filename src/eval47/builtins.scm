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

(defun (application-start)
  (define pair (cons 114 514))
  (dbg-int (car pair))
  (dbg-int (cdr pair))
)
