(defun (--pr47-builtin-cons fst snd)
    (lambda (choice)
        (cond ((= choice 0) fst)
              ((= choice 1) snd)
              (else (dbg-int choice)))
    )
)

(defun (require x) false)

(defun (cons x y) (--pr47-builtin-cons x y))

(defun (car pair) (pair 0))
(defun (cdr pair) (pair 1))
