set -eu

star='*'
space=' '
a=(')' 'b b' c)

multiarg1() {
	local case foo=$star bar=$space baz=(${a[@]})
	case=
	test "$foo-$bar" = '*- '
	test "$(printf '%s, ' "${baz[@]}")" = '), b b, c, '
}
multiarg1
! test -v case
! test -v foo
! test -v bar
! test -v baz

multiarg2() {
	declare case foo=$star bar=$space baz=(${a[@]})
	case=
	test "$foo-$bar" = '*- '
	test "$(printf '%s, ' "${baz[@]}")" = '), b b, c, '
}
multiarg2
! test -v case
! test -v foo
! test -v bar
! test -v baz

foo=$star bar=$space baz=(${a[@]})
test "$foo-$bar" = '*- '
test "$(printf '%s, ' "${baz[@]}")" = '), b b, c, '
unset foo bar baz

readonly case foo=$star bar=$space baz=(${a[@]})
# case=
test "$foo-$bar" = '*- '
test "$(printf '%s, ' "${baz[@]}")" = '), b b, c, '
