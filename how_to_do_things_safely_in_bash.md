Safe ways to do things in bash
==============================

Like programming in C or driving a car,
contemporary shellscript languages require some knowledge and discipline to use safely,
but that's not to say it can't be done.

Purpose of this guide
---------------------

This guide accompanies Shellharden, the corrective syntax highlighter.

Shellharden suggests, and can apply, changes to remove vulnerabilities in shellscripts. This is in accordance with [ShellCheck](https://github.com/koalaman/shellcheck/) and [BashPitfalls](http://mywiki.wooledge.org/BashPitfalls) – Shellharden shall not disagree with these.

The problem is that not all scripts will work with their vulnerabilities simply removed, because *that* was their working principle, and must be rewritten quite differently.
Thus the need for a human in the loop and a holistic methodology.

Why focus on bash?
------------------

This guide is here to show that bash *can* be used safely.

It is the goal and realization of this methodology that
all bash scripts are possible to rewrite into wellformedness,
a representation free of those idiomatic bugs that the language otherwise practically imposes.
This is because the set of bad language features is finite, and each has a substitute.

Unfortunately, [it is hard to defend the correct way of doing something when it isn't also the seemingly simplest][Jussi's two ways].
With this in mind, the python manifesto (`python3 -c 'import this'`),
which says that there should only be one obvious way to do things, and that "explicit is better than implicit",
makes a lot of sense.
While that says something about the impossibility of convincing the vast number of users to adopt a safe methodology,
it is nevertheless possible for those who care.

Clearly, bash is a bad choice, but other prevalent alternatives are not better:

* POSIX shell (a language subset that many shells support) lacks arrays. → Disqualified.
    * Hereunder: dash, busybox ash
* Fish is a relief – easy to use correctly, but (still) lacks a strict mode. → Disqualified.
* Zsh is largely compatible with Bash. → Also qualifies.

### What about non-shellscript languages?

That is in principle the wrong question. Always use the right tool for the job™.
Shellscript languages are languages for running programs, and for using that as a building block.
That is a domain of its own.

This is by no means a defense of shellscripting.
Shellscripts keep getting written, and this is how to do it safely.
However, there is one greater sin than writing something that is obviously a shellscript.
When you know you have a shellscript, you know what to worry about, you can bring in the right expertise, and you have the full arsenal of shell linters.
Not so much if [implicitly invoking the shell with improper quoting](#how-to-avoid-invoking-the-shell-with-improper-quoting).

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

* variables of invariably numeric content: `$?`, `$$`, `$!`, `$#` and array length `${#array[@]}`
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

Variable substitution: This is not the controversy, but just to get it out of the way: These braces are of course needed:

    "${image%.png}.jpg"

String interpolation: Braces also have this role:

    "${var}string literal"

When expanding a variable inside a string, a closing brace can be used to delimit the end of the variable name from subsequent characters of the string literal.
This makes a difference if and only if the next character can take part in a variable name, in other words, if it is an identifier tail character, in regex `[_0-9a-zA-Z]`.

Strictly speaking, it never hurts to always use braces. Does that make it a good idea?

Note that this is not a question of correctness, but of brittleness
(the script would have to be edited, and a mistake be made, before it becomes incorrect).
Strictly speaking not that either, because the problem itself is unnecessary:
Quotes are obligatory anyway; just quote variables individually to avoid the problem
(quotes can always replace braces, but not the opposite, and you never need both).
The result is a concatenation instead of interpolation:

    "$var"'string literal'

Now that the question is clear, your author would say: Mostly not.
In terms of which way to go for consistency's sake, considert that
most variable expansions aren't interpolations. And they shouldn't:
The noble thing to do for a shellscript (or any glue code) is to pass arguments cleanly.
Let's focus on passing arguments cleanly:

* Bad: `some_command $arg1 $arg2 $arg3`
* Bad and verbose: `some_command ${arg1} ${arg2} ${arg3}`
* Good but verbose: `some_command "${arg1}" "${arg2}" "${arg3}"`
* Good: `some_command "$arg1" "$arg2" "$arg3"`

The braces don't do anything objectively good here.

In your author's experience, there is rather a negative correlation between unnecessary use of braces and proper use of quotes – nearly everyone chooses the "bad and verbose" instead of "good but verbose" form! My speculations:

* Fear of the wrong thing: Instead of worrying about the real danger (missing quotes), a beginner might worry that a variable named `$prefix` would influence the expansion of `"$prefix_postfix"` – this is simply not how it works.
* Cargo cult – writing code in testament to the wrong fear perpetuates it.
* Braces compete with quotes under the limits of tolerable verbosity.

Shellharden will add and remove braces on an as-needed basis when it needs to add quotes:

`${arg} $arg"ument"` → `"$arg" "${arg}ument"`

It will also remove braces on individually quoted variables:

`"${arg}"` → `"$arg"`

As of Shellharden 4.3.0, braces are allowed in string interpolations (not that it adds them):

    "${var} "
    " ${var}"
    "${var}${var}"

Thus, a neutral stance in interpolations
(being not a style formatter, Shellharden is not supposed to make subjective changes).
Previously, it rewrote interpolations too on an as-needed basis,
but as noted here, this could indeed be relaxed.

#### Gotcha: Numbered arguments

Unlike normal *identifier* variable names (in regex: `[_a-zA-Z][_a-zA-Z0-9]*`), numbered arguments require braces (string interpolation or not). ShellCheck says:

    echo "$10"
          ^-- SC1037: Braces are required for positionals over 9, e.g. ${10}.

This was deemed too subtle to either fix or ignore: Shellharden will print a big error message and bail if it sees this.

Since braces are required above 9, Shellharden permits them on all numbered arguments.

Use arrays FTW
--------------

In order to be able to quote all variables, you must use real arrays when that's what you need, not whitespace delimited strings.

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

Look how similar the two examples are: There is no algorithmical difference between using real arrays instead of strings as a (bad) substitute.
A bonus point goes to the array syntax for not needing line continuations, making those lines possible to comment.
They are not equivalent, of course, as the "bad" example uses a whitespace delimited string,
which breaks down as soon as a filename contains whitespace, and risks deleting the wrong files.

Is the second example fixable? In theory, yes; in practice, no.
While it is *possible* to represent a list in a string,
even approachable if a suitable delimiter is known,
it becomes hairy (escaping and unescaping the delimiter) to do 100% generically correct.
Worse, getting it back into array form can not be abstracted away (try `set -- a b c` in a function).
The final blow is that fighting such an abstraction failure of the language is pointless if you can choose a different language.

Arrays is the feature that becomes absurdly impractical to program correctly without. Here is why:
* You need *some* datastructure, that can take zero or more values, for passing zero or more values around cleanly.
* In particular, [command arguments are fundamentally arrays](http://manpag.es/RHEL6/3p+exec). Hint: Shell scripting is all about commands and arguments.
* All POSIX shells secretly support arrays anyway, in the form of the argument list `"$@"`.

The recommendation of this guide must therefore be to not give POSIX compatibility a second thought.
The POSIX shell standard is hereby declared unfit for our purposes.
Likewise, sadly, for minimalistic POSIX compatible shells like [Dash](https://wiki.ubuntu.com/DashAsBinSh#A.24.7B....7D) and Ash that don't support arrays either.
As for Zsh, it supports a superset of Bash's array syntax, so it is good.

The lack of a minimalistic shell with array support is a bummer for embedded computuers, where shipping another language is cost sensitive, yet expectations for safety are high. Busybox is impressive for what you get in a small size, but as part of it, you get Ash, which is a hair puller.

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

### hashbang

    #!/usr/bin/env bash

* Portability consideration: The absolute path to `env` is likely more portable than the absolute path to `bash`. Case in point: [NixOS](https://nixos.wiki/wiki/NixOS). POSIX mandates [the existence of `env`](http://manpag.es/RHEL6/1p+env), but bash is not a posix thing.
* Safety consideration: No language flavor options like `-euo pipefail` here! It is not actually possible when using the `env` redirection, but even if your hashbang begins with `#!/bin/bash`, it is not the right place for options that influence the meaning of the script, because it can be overridden, which would make it possible to run your script the wrong way. However, options that don't influence the meaning of the script, such as `set -x` would be a bonus to make overridable (if used).

### Safer and better globbing

    shopt -s nullglob globstar

* `nullglob` is what makes `for f in *.txt` work also when zero files happen to match the expression. It removes a special case in the default behavior:
    * The default behavior (unofficially called [passglob](https://github.com/fish-shell/fish-shell/issues/2394#issuecomment-182047129)) is to pass the pattern as-is in that event.
    As always, special cases are an enemy of correctness: It creates a two-sided source of bugs that likes to defy test coverage:
    On one side, it necessitates workarounds when you wanted the general behavior (file existence checks in this case);
    on the other side, it supports a convenient and wrong use case ([nothing is worse than the intersection between convenient and wrong][Jussi's two ways]).
    When you mean to pass the pattern literally, the safe thing to do is to just do that instead: Quote it.
    * `failglob` is also a fine alternative, but not as generally usable:
    It can be used if zero matches would always be an error (and conveniently makes it so),
    whereas `nullglob` makes it both non-special and easy to check for
    (`txt_files=(*.txt); test "${#txt_files[@]}" -eq 0`).
    Also, `failglob` depends on `errexit` (aka. `set -e`) to actually exit on failure.
* `globstar` enables recursive globbing. Since globbing is easier to use correctly than `find`, use it.

### Strict Mode – safe and relevant subset edition

    if test "$BASH" = "" || "$BASH" -uc "a=();true \"\${a[@]}\"" 2>/dev/null; then
        # Bash 4.4, Zsh
        set -euo pipefail
    else
        # Bash 4.3 and older chokes on empty arrays with set -u.
        set -eo pipefail
    fi

This is [Bash's unofficial strict mode](http://redsymbol.net/articles/unofficial-bash-strict-mode/) except:

* `nounset` (aka. `set -u`) is behind a feature check.
* Setting IFS to something safer (but still unsafe): Doesn't hurt, but is irrelevant: Being shellcheck/shellharden compliant means quoting everything – implicit use of IFS is forbidden anyway.

As it turns out, `nounset` **is dangerous** in Bash 4.3 and earlier: In those versions, it [treats empty arrays as unset](http://stackoverflow.com/questions/7577052/bash-empty-array-expansion-with-set-u). What have we just learned about special cases? They are an enemy of correctness. Also, this can't be worked around, and since using arrays is rather basic in this methodology, and they definitely need to be able to hold empty values, this is far from an ignorable problem. If using `nounset` at all, make sure to use Bash 4.4 or another sane shell like Zsh (easier said than done if you are writing a script and someone else is using it). Fortunately, what works with `nounset` will also work without (unlike `errexit`). Thus why putting it behind a feature check is sane at all.

Other alternatives:

* Setting IFS (the *internal field separator*) to the empty string disables word splitting. Sounds like the holy grail, but isn't: Firstly, empty strings still become empty arrays (very uncool in expressions like `test $x = ""`) – you still need to quote everything that *can* be an empty string, and for purposes of static verification, *everything* can (with the exception of a handful of special variables). Secondly, indirect pathname expansion is still a thing (can be turned off, se the next point). Thirdly, it interferes with commands like `read` that also use it, breaking constructs like `cat /etc/fstab | while read -r dev mnt fs opt dump pass; do printf '%s\n' "$fs"; done'`.
* Disabling pathname expansion (globbing) altogether: If there was an option to only disable indirect pathname expansion, I would. Giving up the unproblematic direct one too, that I'm saying you should want to use, is a tradeoff that I can't imagine being necessary, yet currently is.

### Assert that command dependencies are installed

[Declaring your dependencies](https://12factor.net/dependencies) has many benefits, but until this becomes statically verifiable, concentrate on uncommon commands here.
This prevents your script from failing for external reasons in hard-to-reach sections of code, such as in error handling or the end of a long-running script.
It also prevents misbehavior such as `make -j"$(nproc)"` becoming a fork bomb.

    require(){ hash "$@" || exit 127; }
    require …
    require …
    require …

Benefits of using `hash` for this purpose are its low overhead and that it gives you an error message in the failure case.
This doesn't check option compatibility, of course, but it's also not forbidden to add feature checks for that.

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

### Should I use double bracket conditions?

That is unimportant, but let's dispel some myths. We are talking about these forms of conditions:

```
test …
[ … ]
[[ … ]]
```

When in doubt, ask your shell:

    > type test
    test is a shell builtin
    > type [
    [ is a shell builtin
    > type [[
    [[ is a shell keyword

* None of them are external commands (in Bash).
* The two first are commands; the third is magic syntax.
* If you are quoting variable expansions and command substitutions – following this guide at all, double bracket conditions solve a problem you don't have – with more syntax.
* Double brackets are not POSIX. Busybox `ash` supports them, but the wrong way.

For pedagogical purposes, the `test` command is the most honest about being a command. Issues like whitespace sensitivity and how to combine them (unambiguously) become self-evident when looked at the right way.

Double bracket conditions also have more features. But they have good POSIX substitutes for the most part:

* Pattern matching (`[[ $path == *.png || $path == *.gif ]]`): This is what `case` is for.
* Logical operators: The usual suspects `&&` and `||` work just fine – outside commands – and can be grouped with group commands: `if { true || false; } && true; then echo 1; else echo 0; fi`.
* Checking if a variable exists (`[[ -v varname ]]`): Yes, this is possibly a killer argument, but consider the programming style of always setting variables, so you don't need to check if they exist.

### Are empty string comparisons any special?

Of course not:
Quoting also works when strings are empty on purpose!
For readability, prefer normal string comparisons.

Good:

    test "$s" != ""
    test "$s" = ""
    [ "$s" != "" ]
    [ "$s" = "" ]

(Many shells also support `==`, but a single equal-sign is posixly correct.)

Bad (readability):

    test -n "$s"
    test -z "$s"
    [ -n "$s" ]
    [ -z "$s" ]

Plain wrong (always true):

    test -n $s
    [ -n $s ]

Shellharden replaces the `-z/-n` flags with their equivalent string comparisons.

### How to check if a variable exists

The correct way to check if a variable exists came with Bash 4.2 (also verified for zsh 5.6.2) and is not a feature of POSIX.
Consider therefore to avoid the problem by always setting variables, so you don't need to check if they exist.

Alternative (POSIX blessed):

    "${var-val}" # default value if unset
    "${var:-val}" # default value if unset or empty

POSIX allows expanding a possibly unset variable (even with `set -u`) by giving a default value.

Good (if you must):

```
# Feature check to fail early on Mac OS with Bash 3.9:
[[ -v PWD ]]

[[ -v var ]]
```

If using this and there is any chance someone might try to run your script with an earlier Bash version, it is best to fail early. The feature check above tests for a variable that we know exists and results in a syntax error and termination in earlier versions.

Bad:

    test -n "$var"
    [ -n "$var" ]
    [[ -n $var ]]
    [[ -n "$var" ]]

These don't distinguish being unset with being empty (as a string or array) and obviously precludes the use of `set -u`.

Recall that the `-z/-n` flags are effectively string comparisons in disguise. As such, they are even less suitable as variable existence checks. Other than to distinguish intent – a fair but not good argument.

### How to combine conditions (unambiguously)

This problem evaporates when realizing that conditions are commands – POSIX already defines this:
The shell syntax for conjunction `&&`, disjunction `||`, negation `!` and grouping `{}` of commands applies to all commands,
and the arguments to those commands can contain shell syntax all they want.

The confusing part is that the `test` or `[` command has operators for the same:
POSIX (man 1p test) defines these as `-a`, `-o`, `!`, and parentheses.
But is everything that POSIX standardizes safe? Hint: POSIX is more about portability.

These operators are ill-conceived because strings can evaluate to operators,
and different operators take different numbers of operands,
which together can lead to desynchronization.
To prevent string content from changing the meaning of the test,
the same standard prescribes that the number of arguments has the highest precedence:
POSIX defines the unambiguous meaning of a test, from 0 to 4 arguments, but not more.
These definitions do not include any conjunctions or disjunctions
(likely because the unambiguity breaks down in the combinatorial explosion).

Bad (unless your shell can unambiguously interpret a 13-argument test):

    test ! -e "$f" -a \( "$s" = yes -o "$s" = y \)
    [ ! -e "$f" -a \( "$s" = yes -o "$s" = y \) ]

Good:

    ! test -e "$f" && { test "$s" = yes || test "$s" = y; }
    ! [ -e "$f" ] && { [ "$s" = yes ] || [ "$s" = y ]; }

### xyes deemed unnecessary

Is this idiom of any use?

    test x"$s" = xyes

1. Not to preserve empty strings: Use quoting.
2. Not to prevent ambiguity in case the variable contains a valid operator: The number of arguments has highter precedence.
3. When used with the AND/OR operators (`-a/-o`), it can prevent ambiguity. However, this use is unnecessary (see above).

Commands with better alternatives
---------------------------------

### echo → printf

As with any command, there must be a way to control its option parsing to prevent it from interpreting data as options.
The standard way to signify the end of options is with a double-dash `--` argument.

Significance of the double-dash `--` argument, explained in error messages:

    > shellharden --hlep
    --hlep: No such option
    > shellharden -- --hlep
    --hlep: No such file or directory

As such, the GNU version of `echo` (both the bash builtin and `command echo`) is fatally flawed.
Unlike the POSIX version, it takes options, yet it offers no way to suppress further option parsing.
(Specifically, it interprets any number of leading arguments as options until the first argument that is not an option.)

The result is that `echo` is not *generally* possible to use correctly.
(It is safe as long as its first non-option character is provably not a dash – we can not just print anything unpredictable like a variable or command substitution; we must first print *some* literal character, that is not the dash, and *then* the unpredictable data!)

In contrast, `printf` is always possible to use correctly (not saying it is easier)
and can do a superset of `echo` (including its bashisms, just without bashisms).

Bad:

    echo "$var"
    echo -n "$var"
    echo -en "$var\r"

    echo "$a" "$b"
    echo "${array[@]}"

Good:

    printf '%s\n' "$var"
    printf '%s' "$var"
    printf '%s\r' "$var"

    printf '%s %s\n' "$a" "$b"
    printf '%s\n' "${array[*]}"

At this point, it gets tempting to redefine `echo` to something sane,
except that overloading existing functionality is generally not a robust and reassuring practice – it breaks unnoticeably.
For verifiability's sake, better leave `echo` forever broken, and call yours something else:

    println() {
        printf '%s\n' "$*"
    }

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

[Jussi's two ways]: <https://web.archive.org/web/20150905075810/http://voices.canonical.com/jussi.pakkanen/2014/07/22/the-two-ways-of-doing-something/> "If there is an easy way to do something, and another, correct way to do the same, programmers will always choose the easy way. As a corollary for language design, the correct thing to do must also be the easiest."
