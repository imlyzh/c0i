(define (application-start)
  (display "please input x: ")
  (define x (read-line))
  (display "please input y: ")
  (define y (read-line))
  (define z (if (strcmp x y)
                114514
                1919810))
  (display "z = " z "\n")
  (if (or (strcmp x "114514")
          (strcmp x "1919810"))
      (display "The number is so stench!\n"))
)

