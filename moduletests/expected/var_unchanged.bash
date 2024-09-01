# Expands to a number
echo $#
echo $?
echo $$
echo $!
echo ${#}
echo ${?}
echo ${$}
echo ${!}
echo ${#array[@]}

echo "$identifier_azAZ09"
echo "$Identifier_azAZ09"
echo "$_identifier_azAZ09"
echo "$0"
echo "$1"
echo "$2"
echo "$3"
echo "$4"
echo "$5"
echo "$6"
echo "$7"
echo "$8"
echo "$9"
echo "$@"
echo "$*"
echo "$-"
echo "$#"
echo "$?"
echo "$$"
echo "$!"

echo " ${identifier_azAZ09}"
echo " ${Identifier_azAZ09}"
echo " ${_identifier_azAZ09}"
echo "${identifier_azAZ09} "
echo "${Identifier_azAZ09} "
echo "${_identifier_azAZ09} "
echo "${identifier_azAZ09}${identifier_azAZ09}"
echo "${Identifier_azAZ09}${Identifier_azAZ09}"
echo "${_identifier_azAZ09}${_identifier_azAZ09}"
echo "${0}"
echo "${1}"
echo "${2}"
echo "${3}"
echo "${4}"
echo "${5}"
echo "${6}"
echo "${7}"
echo "${8}"
echo "${9}"
echo "${1}0"
echo "${10}"
echo "${@}"
echo "${*}"
echo "${-}"
echo "${#}"
echo "${?}"
echo "${$}"
echo "${!}"

echo "${#array[@]}"
echo "${array[0]}"
echo "${array[@]}"
echo "${array[*]}"
echo "${subst##*/}"
echo "${subst#*/}"
echo "${subst%/*}"
echo "${subst%%/*}"

option2='abc[<{().[]def[<{().[]ghi'
option2=${option2%%[<{().[]*}
test "$option2" = abc && echo yes || echo no

option2='abc[<{().[]def[<{().[]ghi'
rm='[<{().[]'
option2=${option2%%${rm}*}
test "$option2" = abc && echo yes || echo no

