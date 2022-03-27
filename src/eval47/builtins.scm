(defun (cons fst snd)
    (lambda (choice)
        (cond ((= choice 0) fst)
              ((= choice 1) snd))
    )
)

(defun (car pair) (pair 0))
(defun (cdr pair) (pair 1))

(defun (application-start)
  (dbg-int (car (cons 114 514)))
)

