# The few places where omitting quotes is ok

# Not that I like exceptions, but legal is legal.
# See "Where you can omit the double quotes":
# https://unix.stackexchange.com/a/68748

# Assignments
asterisk=$(echo '*')
spacestar=$IFS
spacestar+=$asterisk
a=(a b)
b=${a[@]}
c=$*

# In the case expression
case $spacestar in
	$' \t\n*')
		echo pass
	;;
	*)
		echo fail
	;;
esac
case $(printf ' \t\n*') in
	$' \t\n*')
		echo pass
	;;
	*)
		echo fail
	;;
esac

# Case arms
case $' \t\n*' in
	$spacestar)
		echo pass
	;;
	*)
		echo fail
	;;
esac
case $' \t\n*' in
	$(printf ' \t\n*'))
		echo pass
	;;
	*)
		echo fail
	;;
esac

# Double brackets
if [[ ${a[@]} == ${b[@]} ]]; then
	echo pass
else
	echo fail
fi

# Numeric content
echo $? + $# - ${#a[@]} = $(($?+$#-${#a[@]}))

# Let's allow backticks where they don't hurt
a=`uname -a`

# Counterexamples
pwd=$(pwd)
pwd+=$(pwd)
files=($(ls))
files+=($(ls))
