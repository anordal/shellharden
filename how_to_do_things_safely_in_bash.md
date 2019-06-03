Safe ways to do things in bash
==============================

Like programming in C or driving a car,
contemporary shellscript languages require some knowledge and discipline to use safely,
but that's not to say it can't be done.

Why focus on bash?
------------------

This guide is here to show that in bash, it *can* be done.
Specifically, those systematic bugs that the language encourages can be eliminated by disciplines that are outlined here.
Realize that Bash is *not* a language where
[the correct way to do something is also the easiest](http://voices.canonical.com/jussi.pakkanen/2014/07/22/the-two-ways-of-doing-something/).

The judgement of alternatives is:

* In POSIX shell (a language subset that many shells support), it can not be done. → Disqualified.
* Fish is a relief – easy to use correctly, but (still) lacks a strict mode. → Disqualified.
* Zsh is largely compatible with Bash. → Also qualifies.

What about non-shellscript languages?

That is the wrong question: This is not a defense of shellscripting.
Shellscripts keep getting written, and this is how to do it safely.
There are valid reasons for writing something in shellscript, and in particular, there is one invalid reason not to.
It all comes down to whether your program fundamentally needs to run other programs –
shellscript languages are languages for running programs.

1. The invalid reason: So you want to remove yourself from shellscripting, but are still (implicitly) invoking the shell anyway – does that count as removing yourself from shellscripting? No: You still have a shellscript on your hands, everything in this document still applies, and the chapter [how to avoid invoking the shell with improper quoting](#how-to-avoid-invoking-the-shell-with-improper-quoting) is especially for you.
2. The attractiveness of shellscripting is that it offers the concisest way to programmatically run other programs – supposedly, they would be the right tool for that job. Well, you be the judge.

Why Shellharden?
----------------

First off, your author recommends following the advice that [ShellCheck](https://github.com/koalaman/shellcheck/) gives you. If you have ambitions of ShellCheck compliance, Shellharden is that tool. Shellharden's rules shall not disagree with ShellCheck.

I wrote that:

> those systematic bugs that the language encourages can be eliminated by disciplines that are outlined here

↑ The premise for Shellharden is that fixing all those bugs is both systematic and absolutely daunting – reviewing the fixes is easier.

The first thing to know about bash coding
-----------------------------------------

If there is anything like a driver's license for safe bash coding,
it must be rule zero of [BashPitfalls](http://mywiki.wooledge.org/BashPitfalls):
**Always use quotes.**

An unquoted variable is to be treated as an armed bomb: It explodes upon contact with whitespace and wildcards. Yes, "explode" as in [splitting a string into an array](http://php.net/manual/en/function.explode.php). Specifically, variable expansions, like `$var`, and also command substitutions, like `$(cmd)`, undergo *word splitting*, whereby the string is split on any of the characters in the special `$IFS` variable, which is whitespace by default. Furthermore, any wildcard characters (`*?`) in the resulting words are used to expand those words to match files on your filesystem (*indirect pathname expansion*). This is mostly invisible, because most of the time, the result is a 1-element array, which is indistinguishable from the original string value.

Quoting inhibits word splitting and indirect pathname expansion, both for variables and command substitutions.

Variable expansion:

* Good: `"$my_var"`
* Bad: `$my_var`

Command substitution:

* Good: `"$(cmd)"`
* Bad: `$(cmd)`

There are exceptions where quoting is not necessary, but because it never hurts to quote, and the general rule is to be scared when you see an unquoted variable, pursuing the non-obvious exceptions is, for the sake of your readers, questionable. It looks wrong, and the wrong practice is common enough to raise suspicion: Enough scripts are being written with broken handling of filenames that whitespace in filenames is often avoided…

The exceptions only matter in discussions of style – feel welcome to ignore them. For the sake of style neutrality, Shellharden does honor a few exceptions:

* variables of invariably numeric content: `$?`, `$#` and array length `${#array[@]}`
* assignments: `a=$b`
* the magical case command: `case $var in … esac`
* the magical context between double-brackets (`[[` and `]]`) – this is a language of its own.

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

This was deemed too subtle to either fix or ignore: Shellharden will print a big error message and bail if it sees this.

Since braces are required above 9, Shellharden permits them on all numbered arguments.

Use arrays FTW
--------------

In order to be able to quote all variables, you must use real arrays when that's what you need, not whitespace separated pseudo-array strings.

The syntax is verbose, but get over it. This bashism single-handedly disqualifies the POSIX shell for the purpose of this guide.

Good:

    files=(
        a
        b
    )
    duplicates=()
    for f in "${files[@]}"; do
        if cmp -- "$f" other/"$f"; then
            duplicates+=("$f")
        fi
    done
    if [ "${#duplicates[@]}" -gt 0 ]; then
        rm -- "${duplicates[@]}"
    fi

Bad:

    files=" \
        a \
        b \
    "
    duplicates=
    for f in $files; do
        if cmp -- "$f" other/"$f"; then
            duplicates+=" $f"
        fi
    done
    if ! [ "$duplicates" = '' ]; then
        rm -- $duplicates
    fi

As the example illustrates, your typical script will still look like itself whether you use proper arrays or pseudo-array strings.
As a bonus, array entries are actually possible to comment.
However, the two versions are not equivalent, as the latter breaks down as soon as a filename contains whitespace.
While it is *possible* to represent a list in a string,
even approachable if a suitable delimiter is known,
it is inhumanely impractical to do correctly in a general way (with escaping and unescaping the delimiter),
and be expected to consistently repeat this excersise for every list.

Here is why arrays are such a basic feature for a shell: [Command arguments are fundamentally arrays](http://manpag.es/RHEL6/3p+exec)
(and shell scripting is all about commands and arguments).
Arrays arise naturally all the time in shellscripting.

It follows that lack of arrays is a blatant feature omission of the POSIX shell standard, and that minimalistic POSIX compatible shells like [Dash](https://wiki.ubuntu.com/DashAsBinSh#A.24.7B....7D) and Ash are not worth pursuing for our purposes.
You could say that a shell that makes it artificially impossible to pass multiple arguments around cleanly is comically unfit for purpose.
As for Zsh, it supports a superset of Bash's array syntax, so it is good.

Awareness needs to be raised that Ash, specifically, is holding us back in the seventies on this front, because as the Busybox shell, it gets used on embedded computers where Bash may not be available.

### Those exceptional cases where you actually intend to split the string

Splitting `$string` on the separator `$sep` into `$array`:

Bad (indirect pathname expansion):

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

The otherwise evil IFS variable has a legitimate use in the `read` command, where it can be used as another way to separate fields without invoking indirect pathname expansion.
IFS is brought into significance by requesting either multiple variables or using the array option to `read`.
By disabling the delimiter `-d ''`, we read all the way to the end.
Because read returns nonzero when it encounters the end, it must be guarded against errexit (`|| true`) if that is enabled.

Split to separate variables:

    IFS="$sep" read -rd '' a b rest < <(printf '%s%s' "$string" "$sep") || true

Split to an array:

    IFS="$sep" read -rd '' -a array < <(printf '%s%s' "$string" "$sep") || true

The 3 corner cases are tab, newline and space – when IFS is set to one of these as above, `read` drops empty fields!
Because this is often useful though, this method makes the bottom of the recommendation list instead of disqualification.

### Corollary: Use while loops to iterate strings and command output

Shellharden won't let you get away with this:

    for i in $(seq 1 10); do
        printf '%s\n' "$i"
    done

The intuitive fix – piping into the loop – is not always cool,
because the pipe operator's right operand becomes a subshell.
Not that it matters for this silly example, but it would surprise many
to find that this loop can't manipulate outside variables:

    seq 1 10 | while read -r i; do
        printf '%s\n' "$i"
    done

To avoid future surprises, the bulk of the code should typically not be the subshell.
This is all right:

    while read -r i; do
        printf '%s\n' "$i"
    done < <(seq 1 10)

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

    require(){ hash "$@" || exit 127; }
    require …
    require …
    require …

This includes:

* The hashbang:
    * Portability consideration: The absolute path to `env` is likely more portable than the absolute path to `bash`. Case in point: [NixOS](https://nixos.wiki/wiki/NixOS). POSIX mandates [the existence of `env`](http://manpag.es/RHEL6/1p+env), but bash is not a posix thing.
    * Safety consideration: No language flavor options like `-euo pipefail` here! It is not actually possible when using the `env` redirection, but even if your hashbang begins with `#!/bin/bash`, it is not the right place for options that influence the meaning of the script, because it can be overridden, which would make it possible to run your script the wrong way. However, options that don't influence the meaning of the script, such as `set -x` would be a bonus to make overridable (if used).
* What we need from [Bash's unofficial strict mode](http://redsymbol.net/articles/unofficial-bash-strict-mode/), with `set -u` behind a feature check. We don't need all of Bash's strict mode because being shellcheck/shellharden compliant means quoting everything, which is a level beyond strict mode. Furthermore, `set -u` **must not be used** in Bash 4.3 and earlier. Because that option, in those versions, [treats empty arrays as unset](http://stackoverflow.com/questions/7577052/bash-empty-array-expansion-with-set-u), which makes arrays unusable for the purposes described herein. With arrays being the second most imporant advice in this guide (after quoting), and the sole reason we're sacrificing POSIX compatibility, that's of course unacceptable: If using `set -u` at all, use Bash 4.4 or another sane shell like Zsh. This is easier said than done if there is a possibility that someone might run your script with an obsolete version of Bash. Fortunately, what works with `set -u` will also work without (unlike `set -e`). Thus why putting it behind a feature check is sane at all. Beware of the presupposition that testing and development happens with a Bash 4.4 compatible shell (so the `set -u` aspect of the script gets tested). If this concerns you, your other options are to give up compatibility (by failing if the feature check fails) or to give up `set -u`.
* `shopt -s nullglob` is what makes `for f in *.txt` work correctly when `*.txt` matches zero files. The default behavior (aka. *passglob*) – pass the pattern as-is if it happens to match nothing – is dangerous for several reasons. As for *globstar*, that enables recursive globbing. Globbing is easier to use correctly than `find`. So use it.
* Assert that command dependencies are installed. [Declaring your dependencies](https://12factor.net/dependencies) has many benefits, but until this becomes statically verifiable, concentrate on uncommon commands here. Your motivation should be to prevent long running scripts from failing right at the end, as well as preventing misbehavior such as `make -j"$(nproc)"` becoming a fork bomb. Benefits of using `hash` for this purpose are its low overhead and that it gives you an error message in the failure case. What it doesn't check is indirect dependencies and compatibility level, but at that point, we want package management.

But not:

    IFS=''
    set -f
    shopt -s failglob

* Setting the *internal field separator* to the empty string disables word splitting. Sounds like the holy grail. Sadly, this is no complete replacement for quoting variables and command substitutions, and given that you are going to use quotes, this gives you nothing. The reason you must still use quotes is that otherwise, empty strings become empty arrays (as in `test $x = ""`), and indirect pathname expansion is still active. Furthermore, messing with this variable also messes with commands like `read` that use it, breaking constructs like `cat /etc/fstab | while read -r dev mnt fs opt dump pass; do printf '%s\n' "$fs"; done'`.
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

If at all using the exit status variable `$?` with errexit, it is of course no substitute for the direct check for command success (otherwise, your script won't live to see this variable whenever it is nonzero). Corollary: The failure case is the only place it makes sense to expand the exit status variable `$?` (because success only has one exit status, which we are checking). A second pitfall is that if we negate the command as part of the check, the exit status will be that of the negated command – a boolean with precisely the useful information removed.

Bad:

    command
    if test $? -ne 0; then
        echo Command returned $?
    fi

Bad:

    if ! command; then
        echo Command returned $?
    fi

Good:

    if command; then
        true
    else
        echo Command returned $?
    fi

Good:

    command || echo Command returned $?

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

* Inside double brackets `[[ ]]`, unquoted variables and command substitutions are safe (from word splitting and indirect pathname expansion). That's a partial solution to a problem we don't have – following this guide implies not doing that anywhere to begin with. If you are, you aren't after shellhardening your scripts.
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

### How to check if a variable exists

A correct way to do this is not a feature of idiomatic POSIX/Bash scripting. Consider avoiding the problem when possible by always setting variables, so you don't need to check if they exist.

You can get a long way by giving variables default values. This works even in busybox:

    : "${var:=defaultvalue}"

    # Or more generally
    var="${var:-defaultvalue}"

But if you must know, the correct way to check if a variable exists came with Bash 4.2 (also verified for zsh 5.6.2):

    [[ -v var ]]

If using this and there is any chance someone might try to run your script with an earlier Bash version, remember to fail early. The feature test approach would be to test, in the beginning of the script, for a variable that we know exists, and terminate if the result is wrong. In this case, we get a syntax error in earlier versions, and termination for free, so it suffices to add this to the beginning section:

    [[ -v PWD ]]

Lastly, don't ever use constructs like `[ -n $var ]` or `[ -z $var ]`. They are fundamentally string comparisons against the empty string, only less readable. However, what matters in this section, is their functional critique:

* A string comparison can't distinguish an unset variable from an empty one. Let alone distinguish the ways it can be empty: Environment variables are just strings, so they may be empty strings, but normal shell variables are really arrays – they can be empty arrays or arrays of empty strings (what you think of as the empty string is indistinguishable from a one-element array).
* Expanding a potentially unset variable obviously precludes the use of `set -u`.

Commands with better alternatives
---------------------------------

### echo → printf

The `echo` command is not generally possible to use correctly – it is safe in *certain* cases.
In contrast, `printf` is always possible to use correctly (not saying it is easier).

The issue is that the bash version of `echo` interprets (any number of) leading arguments as options (until the first argument that is not an option),
with no way to suppress further option parsing (as usually signified with a double-dash `--` argument).
As with any command, safe use requires control of its option parsing (you don't want it to interpret your data as options).
In `echo`'s case, we are safe as long as its first non-option character is provably not a dash – we can not just print anything unpredictable (like a variable or command substitution) – we must first print *some* literal character, that is not the dash, and *then* the unpredictable data!

Even if you actually want to use `echo`s options, be aware that they are a bashism, and and that the more portable `printf` command can do all that and more.

Bad:

    echo "$var"
    echo -n "$var"
    echo -e "\e[1;33m$var\e[m"

Good:

    printf '%s\n' "$var"
    printf '%s' "$var"
    printf '\e[1;33m%s\e[m\n' "$var"

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
