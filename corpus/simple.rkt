===
boolean
===

#t
#f
#true
#false

---
(program
  (boolean)
  (boolean)
  (boolean)
  (boolean))

===
number
===

-1
1/2
1.0
1+2i
1/2+3/4i
1.0+3.0e7i
2e5
#i5
#e2e5
#x2e5
#b101
+inf.t
-nan.t

---

(program
  (number)
  (number)
  (number)
  (number)
  (number)
  (number)
  (number)
  (number)
  (number)
  (number)
  (number)
  (number)
  (number))

===
character
===

#\newline
#\n
#\u3BB
#\λ

---
(program
  (character)
  (character)
  (character)
  (character))

===
symbol
===

Apple
Ap#ple
Ap ple
Ap| |ple
Ap\ ple
#%Apple
123app
app123app123
123app123
中文

---
(program
  (symbol)
  (symbol)
  (symbol)
  (symbol)
  (symbol)
  (symbol)
  (symbol)
  (symbol)
  (symbol)
  (symbol)
  (symbol))
