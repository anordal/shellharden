# Not done

## Errata

(boring, not a priority: please open an issue if it matters)

Unimplemented nesting:
* variables: "${var-:"$()"}"
* [[ "$()" ]]
* math: $((1+1+$((1))))

## Nice haves:
* --allow-var-braces=interpolation,strict
* -c 'check this'
* Rewrite alias to function
* Rewrite the xyes antipattern

## Write about:
* errexit → errtrace ?
* Gotcha: Command substitution "$()" trims whitespace
* Useless uses of find
* cp file dir → cp file dir/
* realpath → readlink -f ?
