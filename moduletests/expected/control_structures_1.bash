let ivar be the test pilot

#
# Ivar is in command position.
#
[[ $ivar ]] && [[ $ivar ]] || ! [[ $ivar ]]; [[ $ivar ]] & [[ $ivar ]] | [[ $ivar ]]

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

\
[[ $ivar ]]

true \
# Comments don't have line continuations. \
[[ $ivar ]]

"$([[ $ivar ]])"
<([[ $ivar ]])
>([[ $ivar ]])
([[ $ivar ]])
{ [[ $ivar ]] } [[ $ivar ]]
f()
{
	[[ $ivar ]]
}
f(){ [[ $ivar ]] }
function f(){ [[ $ivar ]] }

oddvar="$(
	case [[ in
		# comment
		[[) # comment
			# comment
			[[ $ivar ]] # comment
			# comment
		;; # comment
		# comment
	esac
)here is where the string continues"

case true$(true)true in
	true$(true)true)([[ $ivar ]]);;
esac
case true"$(true)"true in
	true"$(true)"true)([[ $ivar ]]);;
esac
