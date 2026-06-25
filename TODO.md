
# Badly handled
* `type=${type}`: Not supposed to be except from cury removal.
* `for type in ${UBOOT_CONFIG}`: Should become an array expansion.

# Features
* -c 'check this' (like bash -c '')
* Perhaps not silently fix definite bugs
  like `[ -n $var ]` and `$10`
  but add an explicit option: --fix-definite-bugs

* --keep-varbraces

# Rewriting
* sort | uniq → sort -u (man 1p sort approves)
* alias → function
* eval is evil: Color blinking red?
* for i in seq → for ((i…))
* for i in … → while read < <(…)

# Code organisation
* reduce perilous boilerplate
  * make flush an error for easier propagation
* approach agreement with rust-fmt

# Write about:
* errexit → errtrace ?
* Gotcha: Command substitution "$()" trims whitespace
* Useless uses of find
* cp file dir → cp file dir/
* realpath → readlink -f ?
