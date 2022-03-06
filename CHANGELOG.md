# Changelog

## 4.2.0

*More helpful with `for` and `test`*

* Variables that must necessarily be arrays in order to be quoted now become
  array expansions instead of simply being quoted
  (they must still manually be changed to arrays):
  * `for i in $a` → `for i in "${a[@]}"`
* Rewriting array serialization to array expansion in contexts where quoting
  is needed, now also for named arrays
  (analogous to `$*` → `"$@"`):
  * `${a[*]}` → `"${a[@]}"`
* `test` command normalization (see the howto for justification):
  * Empty string tests:
    * `test -n "$s"` → `test "$s" != ""`
    * `test -z "$s"` → `test "$s" = ""`
  * xyes deemed unnecessary:
    * `test x"$s" = xyes` → `test "$s" = yes`
* Bugfix: Fix lookahead on short read. See commit 21904cce1.

## 4.1.3

*A pandemy's worth of maintenance*
(when "Covid" stopped being a perfect name for a video conferencing company)

* Syntactic fixes
  * Fix nested case (#37)
  * Recognise nested variable expansion (#39)
  * Recognise arithmetic statement (#42)
  * Recognise export assignments (like local, declare, readonly)
  * Allow unquoted $* in contexts where quoting is not required, such as s=$*
  * Recognise negation, or rather that what follows is in command position,
    as needed to recognise assignments like || ! s=$i in loop conditons.
  * \`pwd\` rewrites to $PWD, also where quoting is not required
    (this was an oversight)
* Feature fixes
  * --check no longer leaks out syntax errors or other error output.
* Testing
  * Make tests run on GitHub
  * Find & test the right build's executable (debug/release), not just both.
  * Test that Shellharden is idempotent and exercise --check on current tests
* Color:
  * Brighten the comment color 3× for readability on IPS screens where dark
    colors look black (or KDE Breeze's not so black terminal background).
  * Change 'single quoted string' from yellow to gold.
  * Use 3-bit background color for terminals that don't support 24-bit color.
    This is merely the most important coloring; the syntax is still
    highlighted in 24-bit color and requires a 24-bit terminal to see.
  * Change color lazily (less work for the terminal).

## 4.1.2

*One refactoring, plenty of necessary fixes*

* Fix old bug: Wrong quoting of associative array assignments (#31)
* More permissive: Allow unquoted arguments to local, declare and readonly (#30)
* Consistency: Rewrite `` `pwd` `` → `$PWD` directly, not via `$(pwd)`
* Compatibility with newer Rust: rustc 1.37 through 1.41 and 2018 edition
* Less code: Collapse nested enums in oft-used return type
* Maintainers: Cargo.lock is now included (#28)

## 4.1.1

*More testing*

* Allow "$*" (no need to rewrite it to "$@" as long as the quotes are on).
* Recognise premature esac to avoid parse error (seen on rustup.sh).
* Write this changelog.
* Cargo Clippy compliant.
* Unittests! Currently focused on corner cases that are hard to moduletest,
  namely lookahead.
* Corner cases in the keyword detection inside the `case` statement were fixed.
  This would manifest as false positive and false negative detection of the `in`
  and `esac` keywords, followed by a likely parse error.
  This stems from version 4.0 and was not seen in the wild AFAIK.
  The most glaring bug was false positive detection when prefixed.
  Less so were the false negatives related to lookahead.
* The special variables $$, $! and $- are now recognized.

## 4.1

*The feature continuation of 4.0*

Allow non-quoting in more contexts exposed by the 4.0 parser.

* Allow unquoted rvalues (the value part of an assignment).
* Allow unquoted switch and case expressions.
* Allow backticks where quoting is unneeded.

## 4.0.1

* Implement the --version option.
* Expand help text to account for the fact that there is no manpage (I gave that up).

## 4.0

*The version with the one big feature*

* Recognise double square brackets as a context where all quoting rules are off.
    * More detailed syntax highlighting to reflect the added parser states, such as keywords and command position

Because not all double square brackets start a double square bracket context
(because they are not in command position),
it is necessary to recognise where commands begin.
Before this, Shellharden kept track of little more than words, quotes and the occasional heredoc.
To track the command position, it is necessary to add a lot more states to the state machine, including control structures, keywords, assignments, redirection, arrays and group commands (hereunder, functions).
The code was also split from one file into many, and I failed to convince git to track most of the code across the split.

Smaller changes:

* Holistic quoting fix: Don't swallow the question sign glob as part of the string.
* Hook the tests into `cargo --test`.
* Implement the -- option.

## 3.2

*The second publicity feedback version*

* Make it build with Cargo (instead of just rustc).
* Add the -h option (as an alias to --help).
* Tests that actually run. Thanks, Robert!
* Fix crash when the file ends prematurely in a heredoc.

## 3.1

*The immediate publicity feedback version*

* Typo fixes
* Add license
* Add the --check and --replace options

## 3.0

*The publicity compatible version*

* Project rename
    * Rename an option for consistency
* Support arithmetic expansion

Hindsight: This release made some headlines and took the project out of obscurity:

* [lobste.rs](https://lobste.rs/s/4jegyk/how_do_things_safely_bash) (by me)
* [Hacker News](https://news.ycombinator.com/item?id=17057596) (not by me)

## 2.0

*The even better version*

* Holistic quoting:
    * $a$b → "$a$b"
    * $a"string" → "${a}string"
* Smaller replacement diff
* Bail, and print a big warning, on multi-decimal numbered args like $10.
* Improved support for heredocs

## 1.0

*The first usable version.* I could have stopped here.

* Modes of operation
* Visible replacements
* Limited support for heredocs

## Commit fe7b3eb

*Proof of concept*

The first usable syntax highlighter that
sneaks in quotes relatively unnoticeably.

* Parsing works except for heredocs
* One mode of operation

## First commit

*Reinvent cat.*
