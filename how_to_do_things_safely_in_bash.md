Safe ways to do things in bash
==============================

Why bash?
---------

Bash has arrays and a safe mode, which may make it just about acceptable under safe coding practices, when used correctly.

Fish is easier to use correctly, but lacks a safe mode. Prototyping in `fish` is therefore a good idea, provided that you know how to translate correctly from fish to bash.

Preface
-------

This guide accompanies ShellHarden, but your author also recommends [ShellCheck](https://github.com/koalaman/shellcheck/): ShellHarden's rules shall not disagree with ShellCheck.

Bash is not a language where [the correct way to do something is also the easiest](http://voices.canonical.com/jussi.pakkanen/2014/07/22/the-two-ways-of-doing-something/). If there is anything like a driver's license for safe bash coding, it must be rule zero of [BashPitfalls](http://mywiki.wooledge.org/BashPitfalls): Always use quotes.

The first thing to know about bash coding
-----------------------------------------

**Quote like a maniac!** An unquoted variable is to be treated as an armed bomb: It explodes upon contact with whitespace. Yes, "explode" as in [splitting a string into an array](http://php.net/manual/en/function.explode.php). Specifically, variable expansions, like `$var`, and also command substitutions, like `$(cmd)`, undergo *word splitting*, whereby the contained string expands to an array by splitting it on any of the characters in the special `$IFS` variable, which is whitespace by default. This is mostly invisible, because most of the time, the result is a 1-element array, which is indistinguishable from the string you expected.

Not only that, but wildcard characters (`*?`) are also expanded. This process happens after word splitting, so that when a resulting word contains any wildcard characters, that word is now a wildcard pattern, expanding to any matching file paths you may happen to have. So this feature actually looks at your filesystem!

Quoting inhibits both word splitting and wildcard expansion, for variables and command substitutions.

Variable expansion:

* Good: `"$my_var"`
* Bad: `$my_var`

Command substitution:

* Good: `"$(cmd)"`
* Bad: `$(cmd)`

There are exceptions where quoting is not necessary, but because it never hurts to quote, and the general rule is to be scared when you see an unquoted variable, pursuing the non-obvious exceptions is, for the sake of your readers, questionable. It looks wrong, and the wrong practice is common enough to raise suspicion: Enough scripts are being written with broken handling of filenames that whitespace in filenames is often avoided…

The only exceptions honored by Shellharden are variables of numeric content, such as `$?`, `$#` and `${#array[@]}`.

### Should I use backticks?

Command substitutions also come in this form:

* Correct: `` "`cmd`" ``
* Bad: `` `cmd` ``

While it is possible to use this style correctly, it looks even more awkward in quotes and is less readable when nested. The consensus around this one is pretty clear: Avoid.

Shellharden rewrites these into the dollar-parenthesis form.

### Should I use curly braces?

Braces are for string interpolation, i.e. usually unnecessary:

* Bad: `some_command $arg1 $arg2 $arg3`
* Bad and verbose: `some_command ${arg1} ${arg2} ${arg3}`
* Good but verbose: `some_command "${arg1}" "${arg2}" "${arg3}"`
* Good: `some_command "$arg1" "$arg2" "$arg3"`

It does not hurt to always use braces, in theory, but in your author's experience, there is a strong negative correlation between unnecessary use of braces and proper use of quotes – nearly everyone chooses the "bad and verbose" instead of "good but verbose" form!

Your author's theories:

* Fear of the wrong thing: Instead of worrying about the real danger (missing quotes), a beginner might worry that a variable named `$prefix` would influence the expansion of `"$prefix_postfix"` – this is simply not how it works.
* Cargo cult – writing code in testament to the wrong fear perpetuates it.
* Braces compete with quotes under the limits of tolerable verbosity.

The decision was made to ban unnecessary use of braces: Shellharden will rewrite all these variants into the simplest good form.

Now onto string interpolation, where braces are actually useful:

* Bad (concatenation): `$var1"more string content"$var2`
* Good (concatentation): `"$var1""more string content""$var2"`
* Good (interpolation): `"${var1}more string content${var2}"`

Concatenation and interpolation are equivalent in bash (even for arrays, which is ridiculous).

Because Shellharden is not a style formatter, it is not supposed to change correct code. This is true of the "good (concatenation)" example: As far as shellharden is concerned, this is the holy (canonically correct) form.

Shellharden currently adds and removes braces on an as-needed basis: In the bad example, var1 becomes interpolated with braces, but braces are not accepted on var2 even in the good (interpolation) case, since they are never needed at the end of a string. The latter requirement may well be lifted.

#### Gotcha: Numbered arguments

Unlike normal *identifier* variable names (in regex: `[_a-zA-Z][_a-zA-Z0-9]*`), numbered arguments require braces (string interpolation or not). ShellCheck says:

    echo "$10"
          ^-- SC1037: Braces are required for positionals over 9, e.g. ${10}.

Shellharden will refuse to fix this (deemed too subtle).

Since braces are required above 9, Shellharden permits them on all numbered arguments.

Use arrays FTW
--------------

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

Here is why arrays are such a basic feature for a shell: [Command arguments are fundamentally arrays](http://manpag.es/RHEL6/3p+exec)
(and shell scripting is all about commands and arguments).
You could say that a shell that makes it artificially impossible to pass multiple arguments around cleanly is comically unfit for purpose.
Some widespread shells in this category include [Dash](https://wiki.ubuntu.com/DashAsBinSh#A.24.7B....7D) and Busybox Ash.
These are minimal POSIX compatible shells –
what good is that when the most important stuff is *not* in POSIX?

### Those exceptional cases where you actually intend to split the string

Example with `\v` as delimiter (note the second occurence):

    IFS=$'\v' read -d '' -ra a < <(printf '%s\v' "$s")

This avoids wildcard expansion, and it works no matter if the delimiter is `\n`. The second occurence of the delimiter preserves the last element if it's empty. For some reason, the `-d` option must come first, so putting the options together as `-rad ''`, which is tempting, doesn't work. Tested with bash 4.2, 4.3 and 4.4.

Alternatively, for bash 4.4:

    readarray -td $'\v' a < <(printf '%s\v' "$s")

How to begin a bash script
--------------------------

Something like this:

    #!/usr/bin/env bash
    if test "$BASH" = "" || "$BASH" -uc "a=();true \"\${a[@]}\"" 2>/dev/null; then
        # Bash 4.4, Zsh
        set -euo pipefail
    else
        # Bash 4.3 and older chokes on empty arrays with set -u.
        set -eo pipefail
    fi
    shopt -s nullglob globstar

This includes:

* The hashbang:
    * Portability consideration: The absolute path to `env` is likely more portable than the absolute path to `bash`. Case in point: [NixOS](https://nixos.wiki/wiki/NixOS). POSIX mandates [the existence of `env`](http://manpag.es/RHEL6/1p+env), but bash is not a posix thing.
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

To use `local`, separate the declaration from the assignment:

    set -e # Fail if nproc is not installed
    local jobs
    jobs="$(nproc)"
    make -j"$jobs"

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

How to avoid invoking the shell with improper quoting
-----------------------------------------------------

When invoking a command from other programming languages, the wrong thing to do is often the easiest: implicitly invoking the shell. If that shell command is static, fine – either it works, or it doesn't. But if your program is doing any kind of string processing to assemble that command, realize that you are **generating a shellscript**! Rarely what you want, and tedious to do correctly:

* quote each argument
* escape relevant characters in the arguments

No matter which programming language you are doing this from, there are at least 3 ways to construct the command correctly. In order of preferece:

### Plan A: Avoid the shell

If it's just a command with arguments (i.e. no shell features like piping or redirection), choose the array representation.

* Bad (python3): `subprocess.check_call('rm -rf ' + path)`
* Good (python3): `subprocess.check_call(['rm', '-rf', path])`

Bad (C++):

    std::string cmd = "rm -rf ";
    cmd += path;
    system(cmd);

Good (C/POSIX), minus error handling:

    char* const args[] = {"rm", "-rf", path, NULL};
    pid_t child;
    posix_spawnp(&child, args[0], NULL, NULL, args, NULL);
    int status;
    waitpid(child, &status, 0);

### Plan B: Static shellscript

If the shell is needed, let arguments be arguments. You might think this was cumbersome – writing a special-purpose shellscript to its own file and invoking that – until you have seen this trick:

* Bad (python3): `subprocess.check_call('docker exec {} bash -ec "printf %s {} > {}"'.format(instance, content, path))`
* Good (python3): `subprocess.check_call(['docker', 'exec', instance, 'bash', '-ec', 'printf %s "$0" > "$1"', content, path])`

Can you spot the shellscript?

That's right, the printf command with the redirection. Note the correctly quoted numbered arguments. Embedding a static shellscript is fine.

The examples run in Docker because they wouldn't be as useful otherwise, but Docker is also a fine example of a command that runs other commands based on arguments. This is unlike Ssh, as we will see.

### Last option: String processing

If it *has* to be a string (e.g. because it has to run over `ssh`), there is no way around it. We must quote each argument and escape whatever characters are necessary to escape within those quotes. The simplest is to go for single quotes, since these have the simplest escaping rules – only one: `'` → `'\''`.

* Bad (python3): `subprocess.check_call('ssh user@host sha1sum ' + path)`
* Bad (python3): `subprocess.check_call(['ssh', 'user@host', 'sha1sum', path])`
* Often correct (python3): `subprocess.check_call(['ssh', 'user@host', "sha1sum '{}'".format(path.replace("'", "'\\''"))])`

Why is the second example bad? Because Ssh is treacherous: If you try to give multiple arguments to ssh, it will do exactly the wrong thing for you – space-concatenate the arguments without quoting.

Why is there no "good" example here, just an "often correct" one? This is ssh's fault: The correct solution depends on user preference at the other end, namely the remote shell, which can be anything. It can be your mother, in principle. Assuming that the remote shell is bash or another POSIX compatible shell, the "often correct" will in fact be correct, but [fish is incompatible on this point](https://github.com/fish-shell/fish-shell/issues/4907).
