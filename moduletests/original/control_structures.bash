let ivar be the test pilot

#
# Ivar is in command position.
#
[[ $ivar ]] && [[ $ivar ]] || [[ $ivar ]]; [[ $ivar ]] & [[ $ivar ]] | [[ $ivar ]]

if [[ $ivar ]]; then [[ $ivar ]]; elif [[ $ivar ]]; then [[ $ivar ]]; else [[ $ivar ]]; fi
if [[ $ivar ]]
then
	[[ $ivar ]]
elif [[ $ivar ]]
then
	[[ $ivar ]]
else
	[[ $ivar ]]
fi

while [[ $ivar ]]; do [[ $ivar ]]; done
while [[ $ivar ]]
do
	[[ $ivar ]]
done

until [[ $ivar ]]; do [[ $ivar ]]; done
until [[ $ivar ]]
do
	[[ $ivar ]]
done

for i in {,}; do [[ $i ]]; done
for i in {,}
do
	[[ $i ]]
done

true \
# Comments don't have line continuations. \
[[ $ivar ]]

"$([[ $ivar ]])"
<([[ $ivar ]])
>([[ $ivar ]])
([[ $ivar ]])
{ [[ $ivar ]] } [[ $ivar ]]
f() { [[ $ivar ]] }
f()
{
	[[ $ivar ]]
}

oddvar="$(
	case true in
		# comment
		true) # comment
			# comment
			[[ $ivar ]] # comment
			# comment
		;; # comment
		# comment
	esac
)here is where the string continues"

#
# Ivar is an argument.
#
true [[ $ivar ]] && true [[ $ivar ]] || true [[ $ivar ]]; true [[ $ivar ]] & true [[ $ivar ]] | true [[ $ivar ]]
[ $ivar ] && [ $ivar ] || [ $ivar ]; [ $ivar ] & [ $ivar ] | [ $ivar ]
test $ivar && test $ivar || test $ivar; test $ivar & test $ivar | test $ivar

if true [[ $ivar ]]; then true [[ $ivar ]]; elif true [[ $ivar ]]; then true [[ $ivar ]]; else true [[ $ivar ]]; fi
if true [[ $ivar ]]
then
	true [[ $ivar ]]
elif true [[ $ivar ]]
then
	true [[ $ivar ]]
else
	true [[ $ivar ]]
fi

echo line continuation \
[[ $ivar ]]

{[[ $ivar ]]}
echo {} [[ $ivar ]]

for i in [[ $ivar ]]; do :; done
select i in [[ $ivar ]]; do break; done

array=(
	[[ $ivar ]]
)
array+=(
	[[ $ivar ]]
)
