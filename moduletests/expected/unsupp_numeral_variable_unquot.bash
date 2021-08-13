echo 
moduletests/original/unsupp_numeral_variable_unquot.bash: Unsupported syntax: Syntactic pitfall
$10
  ^
This does not mean what it looks like. You may be forgiven to think that the full string of numerals is the variable name. Only the fist is.

Try this and be shocked: f() { echo "$9" "$10"; }; f a b c d e f g h i j

Here is where braces should be used to disambiguate, e.g. "${10}" vs "${1}0".

Syntactic pitfalls are deemed too dangerous to fix automatically
(the purpose of Shellharden is to fix brittle code – code that mostly does what it looks like, as opposed to code that never does what it looks like):
* Fixing what it does would be 100% subtle and might slip through code review unnoticed.
* Fixing its look would make a likely bug look intentional.
