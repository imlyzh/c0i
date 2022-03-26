(defun (curry f x)
  (lambda (y)
    (f x y)))
