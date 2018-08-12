Safe ways to do things in bash
==============================

Why bash?
---------

Sometimes, a shellscript is the right tool for the job.
At that, Bash has arrays and a safe mode,
which in large part makes it possible and practical (respectively) to write scripts without glaring bugs.
Other times, you are invoking the shell because that's what you get.

Fish is easier to use correctly, but lacks a safe mode. Prototyping in `fish` is therefore a good idea, provided that you know how to translate correctly from fish to bash.

Preface
-------

This guide accompanies Shellharden, but your author also recommends [ShellCheck](https://github.com/koalaman/shellcheck/): Shellharden's rules shall not disagree with ShellCheck.

Bash is not a language where [the correct way to do something is also the easiest](http://voices.canonical.com/jussi.pakkanen/2014/07/22/the-two-ways-of-doing-something/). If there is anything like a driver's license for safe bash coding, it must be rule zero of [BashPitfalls](http://mywiki.wooledge.org/BashPitfalls): Always use quotes.

The first thing to know about bash coding
-----------------------------------------

**Quote like a maniac!** An unquoted variable is to be treated as an armed bomb: It explodes upon contact with whitespace and wildcards. Yes, "explode" as in [splitting a string into an array](http://php.net/manual/en/function.explode.php). Specifically, variable expansions, like `$var`, and also command substitutions, like `$(cmd)`, undergo *word splitting*, whereby the string is split on any of the characters in the special `$IFS` variable, which is whitespace by default. Furthermore, any wildcard characters (`*?`) in the resulting words are used to expand those words to match files on your filesystem (*indirect wildcard expansion*). This is mostly invisible, because most of the time, the result is a 1-element array, which is indistinguishable from the original string value.

Quoting inhibits word splitting and indirect wildcard expansion, both for variables and command substitutions.

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

While it is possible to use this style correctly, it is harder: [Backticks require escaping when nested, and examples in the wild are improperly quoted more often than not](http://wiki.bash-hackers.org/scripting/obsolete).

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

For our purposes, lack of arrays is the ~~most~~ pressing feature omission of the POSIX shell standard.
In other words, POSIX is not the standard. The Bash array syntax is, and Zsh supports a superset of it.
Awareness needs to be raised that minimal POSIX compatible shells
like [Dash](https://wiki.ubuntu.com/DashAsBinSh#A.24.7B....7D) and Busybox Ash
are only doing us a disfavor.

### Those exceptional cases where you actually intend to split the string

Splitting `$string` on the separator `$sep` into `$array`:

Bad (indirect wildcard expansion):

    IFS="$sep"
    array=($string)

Good:

    array=()
    while read -rd "$sep" i; do
        array+=("$i")
    done < <(printf '%s%s' "$string" "$sep")

This works for any separator byte (no UTF-8 or multi-character separator string) except NUL. To make it work for NUL, hardcode the literal `$'\0'` in place of `$sep`.

The reason for appending the separator to the end is that the field separator is really a field *terminator* (postfix, not infix). The distinction matters to the notion of an empty field at the end. Skip this if your input is already field terminated.

Alternatively, for Bash 4:

    readarray -td "$sep" array < <(printf '%s%s' "$string" "$sep")

The same notes apply to readarray (hardcoding of NUL, already field terminated input):

    readarray -td $'\0' array < <(find -print0)

Readarray gets a small minus point for only working with ASCII separators (still no UTF-8).

If the separator consists of multiple bytes, it is also possible to do this correctly by string processing (such as by [parameter substitution](https://www.tldp.org/LDP/abs/html/parameter-substitution.html#PSOREX2)).

#### An alternative with 3 corner cases

The otherwise evil IFS variable has a legitimate use in the `read` command, where it can be used as another way to separate fields without invoking indirect wildcard expansion.
IFS is brought into significance by requesting either multiple variables or using the array option to `read`.
By disabling the delimiter `-d ''`, we read all the way to the end.
Because read returns nonzero when it encounters the end, it must be guarded against errexit (`|| true`) if that is enabled.

Split to separate variables:

    IFS="$sep" read -rd '' a b rest < <(printf '%s%s' "$string" "$sep") || true

Split to an array:

    IFS="$sep" read -rd '' -a array < <(printf '%s%s' "$string" "$sep") || true

The 3 corner cases are tab, newline and space – when IFS is set to one of these as above, `read` drops empty fields!
Because this is often useful though, this method makes the bottom of the recommendation list instead of disqualification.

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

How to end a bash script
------------------------

Goal: The script's exit status should convey its overall success or failure.

Reality: The script's exit status is that of the last command executed.

There is a wrong way to end a bash script:
Letting a command used as a condition be the last command executed, so that the script "fails" iff the last condition is false.
While that might happen to be correct for a script, it is a way to encode the exit status that looks accidental and is easily broken by adding or removing code to the end.

The rightness criterion here is that the last statement follows the "Errexit basics" below. When in doubt, end the script with an explicit exit status:

    exit 0

How to use errexit
------------------

Aka `set -e`.

### Errexit basics

Background: If a command that is not used as a condition returns nonzero, the interpreter exits at that point.

Failure is trivial to suppress:

    command || true

Don't skimp on if-statements. You can't use `&&` as a shorthand if-statement without always using `||` as an else-branch. Otherwise, the script terminates if the condition is false.

Bad:

    command && …

Good (contrived):

    command && … || true

Good (contrived):

    ! command || …

Good (idiomatic):

    if command; then
        …
    fi

To capture a command's output while using it as a condition, use an assignment as the condition (but see below on not using `local` on assignments):

    if output="$(command)"; then
        …
    fi

If at all using the exit status variable `$?`, it must be conditioned on the command (specifically, it only makes sense to use in the conditional branch where it is nonzero). Otherwise, your script won't live to see this variable whenever it is nonzero.

Bad:

    command
    if test $? -ne 0; then
        echo Command returned $?
    fi

Good:

    if ! command; then
        echo Command returned $?
    fi

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
    make -j"$jobs"

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

Sometimes, POSIX is cruel. Errexit is ignored in functions, group commands and even subshells if the caller is checking its success. These examples all print `Unreachable` and `Great success`, despite all sanity.

Subshell:

    (
        set -e
        false
        echo Unreachable
    ) && echo Great success

Group command:

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

How to write conditions
-----------------------

### Should I use double brackets?

What for? It does not matter.

Issue: `test`, `[` and `[[` are largely interchangeable.

If you are following this guide, the usual arguments don't apply:

* Inside double brackets `[[ ]]`, unquoted variables and command substitutions are safe (from word splitting and indirect wildcard expansion). That's a partial solution to a problem we don't have – following this guide implies not doing that anywhere to begin with. If you are, you aren't after shellhardening your scripts.
* The usual counterargument is POSIX compatibility. We sacrificed that for arrays.

Other concerns:

*—What if I have n00b contributors?*

This argument goes both ways: `[[` has a more forgiving syntax because it *is* syntax, not a command. **Quoting is required everywhere else.** The fewer exceptions, the lesser confusion. If you want to be pedagogical, use the `test` command – it is honest about being a command, not syntax.

*—What if `[[` has a feature I need?*

Chances are that you don't know the substitute.

* Pattern matching (`[[ $path == *.png || $path == *.gif ]]`): This is what `case` is for.
* Logical operators: The usual suspects `&&` and `||` work just fine – outside commands – and can be grouped with group commands: `if { true || false; } && true; then echo 1; else echo 0; fi`.
* Checking if a variable exists (`[[ -v varname ]]`): Yes, this is possibly a killer argument, but consider the programming style of always setting variables, so you don't need to check if they exist.

Shellharden will not stop you from using quotes in syntactical contexts where it does not matter, but that would deviate from common practice.

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

A very typical filename, in single quotes:

    echo 'Don'\''t stop (12" dub mix).mp3'

Now, how to use this trick to run commands safely over ssh? It's impossible! Well, here is an "often correct" solution:

* Often correct (python3): `subprocess.check_call(['ssh', 'user@host', "sha1sum '{}'".format(path.replace("'", "'\\''"))])`

The reason we have to concatenate all the args to a string in the first place, is so that Ssh won't do it the wrong way for us: If you try to give multiple arguments to ssh, it will treacherously space-concatenate the arguments without quoting.

The reason this is not generally possible is that the correct solution depends on user preference at the other end, namely the remote shell, which can be anything. It can be your mother, in principle. Assuming that the remote shell is bash or another POSIX compatible shell, the "often correct" will in fact be correct, but [fish is incompatible on this point](https://github.com/fish-shell/fish-shell/issues/4907).

#### How to be Fish compatible

This is only necessary if you are forced to interoperate with a user's favourite shell, such as when implementing [ssh-copy-id](https://github.com/fish-shell/fish-shell/issues/2292).

The issue with supporting Fish is that the subset of common syntax with POSIX/Bash is mostly useless.
The general approach is therefore to duplicate the code – obviously against any safety recommendation.

But if you must, so be it:

    test '\'

    echo "This is POSIX!"

    test '

    echo "This is fish!"

    test \'
