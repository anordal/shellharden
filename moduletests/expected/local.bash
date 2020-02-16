set -eu

star='*'
space=' '
a=(')' 'b b' c)

multiarg1() {
	local foo=$star bar=$space baz=("${a[@]}") false
	test "$foo-$bar" = '*- '
	test "$(printf '%s, ' "${baz[@]}")" = '), b b, c, '
	false=
}
multiarg1
! test -v foo
! test -v bar
! test -v baz
! test -v false

multiarg2() {
	declare foo=$star bar=$space baz=("${a[@]}") false
	test "$foo-$bar" = '*- '
	test "$(printf '%s, ' "${baz[@]}")" = '), b b, c, '
	false=
}
multiarg2
! test -v foo
! test -v bar
! test -v baz
! test -v false

foo=$star bar=$space baz=("${a[@]}")
test "$foo-$bar" = '*- '
test "$(printf '%s, ' "${baz[@]}")" = '), b b, c, '
unset foo bar baz

readonly foo=$star bar=$space baz=("${a[@]}") false
test "$foo-$bar" = '*- '
test "$(printf '%s, ' "${baz[@]}")" = '), b b, c, '

