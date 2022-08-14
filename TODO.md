# Not done

## Nice haves:
* -c 'check this'
* Things that never work: $10 and [ -n $var ]:
  Fail by default, add --unbreak/--fix-neverworking

# Language features
* alias → function
* eval is evil: Color blinking red?
* for i in seq → for ((i…))
* for i in … → while read < <(…)

## Code organisation
* reduce perilous boilerplate
  * make flush an error for easier propagation
* approach agreement with rust-fmt

## Write about:
* errexit → errtrace ?
* Gotcha: Command substitution "$()" trims whitespace
* Useless uses of find
* cp file dir → cp file dir/
* realpath → readlink -f ?
