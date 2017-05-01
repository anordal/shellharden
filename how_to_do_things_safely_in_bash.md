Safe ways to do things in bash
==============================

Why bash?
---------

Bash has arrays and a safe mode, which may make it just about acceptable under safe coding practies, when used correctly.

Fish is easier to use correctly, but lacks a safe mode. Prototyping in `fish` is therefore a good idea, provided that you know how to translate correctly from fish to bash.

The first thing to know about bash coding
-----------------------------------------

**Quote like a maniac!** An unquoted variable is to be treated as an armed bomb: It explodes upon contact with whitespace. Yes, I mean "explode" as in [splitting a string into an array](http://php.net/manual/en/function.explode.php). Specifically, variable expansions, like `$var`, and also command substitutions, like `$(cmd)`, undergo *word splitting*, whereby the contained string expands to an array by splitting it on any of the characters in the special `$IFS` variable, which is whitespace by default. This is mostly invisible, because most of the time, the result is a 1-element array, which is indistinguishable from the string you expected.

Not only that, but wildcard characters (`*?`) are also expanded. This process happens after word splitting, so that when a resulting word contains any wildcard characters, that word is now a wildcard pattern, expanding to any matching file paths you may happen to have. So this feature actually looks at your filesystem!

Quoting inhibits both word splitting and wildcard expansion, for variables and command substitutions.

Good: `"$my_var"`
Bad: `$my_var`

Special remark: Using curly braces is no substitute for quoting, so don't pretend it is:

Extra bad (adds confusion): `${my_var}`

Good command substitution: `"$(cmd)"`
Bad command substitution: `$(cmd)`

Good (but unusual) command substitution: `` "`cmd`" ``
Bad command substitution: `` `cmd` ``

There are exceptions where quoting is not necessary, but because it never hurts to quote, and the general rule is to be scared when you see an unquoted variable, pursuing the non-obvious exceptions is, for the sake of your readers, questionable. It looks wrong, and the wrong practice is common enough to raise suspicion: Enough scripts are being written with broken handling of filenames that whitespace in filenames is often avoided…

The only exceptions honored by Naziquote, are variables of numeric content, such as `$?`, `$#` and `${#array[@]}`.

### Use arrays FTW

In order to be able to quote all variables, you must use real arrays when that's what you need, not whitespace separated pseudo-array strings.

The syntax is verbose, but get over it. This bashism is reason alone to drop posix compatibility for most shellscripts.

Good:

    array=(
        a
        b
    )
    array+=(c)
    if [ ${#array[@]} -gt 0 ]; then
        rm -- "${array[@]}"
    fi

Bad:

    pseudoarray=" \
        a \
        b \
    "
    pseudoarray="$pseudoarray c"
    if ! [ "$pseudoarray" = '' ]; then
        rm -- $pseudoarray
    fi

### Those exceptional cases where you actually intend to split the string

Example with `\v` as delimiter (note the second occurence):

    IFS=$'\v' read -d '' -ra a < <(printf '%s\v' "$s")

This avoids wildcard expansion, and it works no matter if the delimiter is `\n`. The second occurence of the delimiter preserves the last element if it's empty. For some reason, the `-d` option must come first, so putting the options together as `-rad ''`, which is tempting, doesn't work. Tested with bash 4.2, 4.3 and 4.4.

Alternatively, for bash 4.4:

    readarray -td $'\v' a < <(printf '%s\v' "$s")

How to open
-----------

    #!/bin/bash
    set -eo pipefail
    IFS=''
    shopt -s nullglob globstar

Line by line, this is:
1. The hashbang: Note: No language flavor options like `-euo pipefail` here! The hashbang is not the right place for options that influence the meaning of the script, because it can be overridden, which would make it possible to run your script the wrong way. However, options that don't influence the meaning of the script, such as `set -x` is just a bonus to make overridable (if used).
2. Bash's unofficial *safe mode*, minus `set -u`. Don't use `set -u` if there is any chance someone might run your script with bash 4.3 or earlier. It was unusable together with arrays (empty arrays were accused of being unbound), unless you fancied [this ugly workaround](http://stackoverflow.com/questions/7577052/bash-empty-array-expansion-with-set-u).
3. Setting the *internal field separator* to the empty string disables word splitting. Note 1: Not the same as unsetting it. Note 2: You *still* must quote variables, as otherwise, empty strings become empty arrays, and wildcard expansion is still active.
4. Let unmatched wildcards expand to empty arrays, as they should, and enable recursive globbing. Globbing is nice in itself (just not as part of variable expansion) because it can replace many uses of `find`, which is why I don't use `set -f`. This behavior is more fish-like, btw.
