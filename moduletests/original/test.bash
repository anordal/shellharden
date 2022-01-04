
test -z `test -z $1`
test -z "$(test -z "$1")"
[ -z `[ -z $1 ]` ]
[ -z "$([ -z "$1" ])" ]

# NB: Unquoted `test -n` is always true.
test -n `test -n $1`
test -n "$(test -n "$1")"
[ -n `[ -n $1 ]` ]
[ -n "$([ -n "$1" ])" ]

# xyes
test x$([ x"$a$b" = x"" ])$b = xyes
test x$([ x"$a$b" != x"" ])$b != x''
test x$([ x"$a$b" == x"" ])$b == x
test x$([ x"$a$b" == "" ])$b == ex
