var=${one:-${two}} ${one:-${two}}
export var=${one:-${two}}

[[ $([[ $(printf '%s\n' a b) == $'a\nb' ]] && echo ja || echo nei) == ja ]]

echo $(($((1))+$(expr 1)+$(calc 1)))
