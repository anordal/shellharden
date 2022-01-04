
test "$(test "$1" = "")" = ""
test "$(test "$1" = "")" = ""
[ "$([ "$1" = "" ])" = "" ]
[ "$([ "$1" = "" ])" = "" ]

# NB: Unquoted `test -n` is always true.
test "$(test "$1" != "")" != ""
test "$(test "$1" != "")" != ""
[ "$([ "$1" != "" ])" != "" ]
[ "$([ "$1" != "" ])" != "" ]

# xyes
test "$([ "$a$b" = "" ])$b" = yes
test "$([ "$a$b" != "" ])$b" != ''
test "$([ "$a$b" == "" ])$b" == ""
test x"$([ x"$a$b" == "" ])$b" == ex
