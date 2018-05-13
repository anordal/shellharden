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

The only exceptions honored by Shellharden are variables of numeric content, such as `$?`, `$#` and `${#array[@]}`.

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

### Gotcha: Numbered arguments

Having argued that enclosing variables in braces for no reason is bad style (adds confusion), this does not extend to numbered arguments. As ShellCheck says:

    echo "$10"
          ^-- SC1037: Braces are required for positionals over 9, e.g. ${10}.

Shellharden will refuse to fix this (deemed too subtle).

Since braces are required above 9, Shellharden permits them on all numbered arguments.

How to begin a bash script
--------------------------

Something like this:

    #!/usr/bin/env bash
    if test "$BASH" = "" || "$BASH" -uc 'a=();true "${a[@]}"' 2>/dev/null; then
        # Bash 4.4, Zsh
        set -euo pipefail
    else
        # Bash 4.3 and older chokes on empty arrays with set -u.
        set -eo pipefail
    fi
    shopt -s nullglob globstar

This includes:

* The hashbang:
    * Portability consideration: The absolute path to `env` is likely more portable than the absolute path to `bash`. Case in point: [NixOS](https://nixos.wiki/wiki/NixOS). POSIX mandates `/bin/sh` and [the existence of `env`](http://manpag.es/RHEL6/1p+env), but bash is not a posix thing. If you want to be 100% covered by posix, give up – wrap it in a posix script instead.
    * Safety consideration: No language flavor options like `-euo pipefail` here! It is not actually possible when using the `env` redirection, but even if your hashbang begins with `#!/bin/bash`, it is not the right place for options that influence the meaning of the script, because it can be overridden, which would make it possible to run your script the wrong way. However, options that don't influence the meaning of the script, such as `set -x` would be a bonus to make overridable (if used).
* What we need from [Bash's unofficial strict mode](http://redsymbol.net/articles/unofficial-bash-strict-mode/), with `set -u` behind a feature check. We don't need all of Bash's strict mode because being shellcheck/shellharden compliant means quoting everything, which is a level beyond strict mode. Furthermore, `set -u` **must not be used** in Bash 4.3 and earlier. Because that option, in those versions, [treats empty arrays as unset](http://stackoverflow.com/questions/7577052/bash-empty-array-expansion-with-set-u), which makes arrays unusable for the purposes described herein. With arrays being the second most imporant advice in this guide (after quoting), and the sole reason we're sacrificing POSIX compatibility, that's of course unacceptable: If using `set -u` at all, use Bash 4.4 or another sane shell like Zsh. This is easier said than done if there is a possibility that someone might run your script with an obsolete version of Bash. Fortunately, what works with `set -u` will also work without (unlike `set -e`). Thus why putting it behind a feature check is sane at all. Beware of the presupposition that testing and development happens with a Bash 4.4 compatible shell (so the `set -u` aspect of the script gets tested). If this concerns you, your other options are to give up compatibility (by failing if the feature check fails) or to give up `set -u`.
* `shopt -s nullglob` is what makes `for f in *.txt` work correctly when `*.txt` matches zero files. The default behavior (aka. *passglob*) – pass the pattern as-is if it happens to match nothing – is dangerous for several reasons. As for *globstar*, that enables recursive globbing. Globbing is easier to use correctly than `find`. So use it.

But not:

    IFS=''
    set -f
    shopt -s failglob

* Setting the *internal field separator* to the empty string disables word splitting. Sounds like the holy grail. Sadly, this is no complete replacement for quoting variables and command substitutions, and given that you are going to use quotes, this gives you nothing. The reason you must still use quotes is that otherwise, empty strings become empty arrays (as in `test $x = ""`), and indirect wildcard expansion is still active. Furthermore, messing with this variable also messes with commands like `read` that use it, breaking constructs like `cat /etc/fstab | while read -r dev mnt fs opt dump pass; do echo "$fs"; done'`.
* Disabling wildcard expansion: Not just the notorious indirect one, but also the unproblematic direct one, that I'm saying you should want to use. So this is a hard sell. And this too should be completely unnecessary for a script that is shellcheck/shellharden conformant.
* As an alternative to *nullglob*, *failglob* fails if there are zero matches. While this makes sense for most commands, for example `rm -- *.txt` (because most commands that take file arguments don't expect to be called with zero of them anyway), obviously, *failglob* can only be used when you are able to assume that zero matches won't happen. That just means you mostly won't be putting wildcards in command arguments unless you can assume the same. But what can always be done, is to use *nullglob* and let the pattern expand to zero arguments in a construct that can take zero arguments, such as a `for` loop or array assignment (`txt_files=(*.txt)`).

How to use errexit
------------------

Aka `set -e`.

### Program-level deferred cleanup

In case errexit does its thing, use this to set up any necessary cleanup to happen at exit.

    tmpfile="$(mktemp -t myprogram-XXXXXX)"
    cleanup() {
        rm -f "$tmpfile"
    }
    trap cleanup EXIT

### Gotcha: Errexit is ignored in command arguments

Here is a nice underhanded fork bomb that I learnt the hard way – my build script worked fine on various developer machines, but brought my company's buildserver to its knees:

    set -e # Fail if nproc is not installed
    make -j"$(nproc)"

Correct (command substitution in assignment):

    set -e # Fail if nproc is not installed
    jobs="$(nproc)"
    make -j "$jobs"

Caution: Builtins like `local` and `export` are also commands, so this is still wrong:

    set -e # Fail if nproc is not installed
    local jobs="$(nproc)"
    make -j"$jobs"

ShellCheck warns only about special commands like `local` in this case.

### Gotcha: Errexit is ignored depending on caller context

Sometimes, POSIX is cruel. Errexit is ignored in functions, scopes and even subshells if the caller is checking its success. These examples all print `Unreachable` and `Great success`, despite all sanity.

Subshell:

    (
        set -e
        false
        echo Unreachable
    ) && echo Great success

Scope:

    {
        set -e
        false
        echo Unreachable
    } && echo Great success

Function:

    f() {
        set -e
        false
        echo Unreachable
    }
    f && echo Great success

This makes bash with errexit practically incomposable – it is *possible* to wrap your errexit functions so that they still work, but the effort it saves (over explicit error handling) becomes questionable. Consider splitting into completely standalone scripts instead.
