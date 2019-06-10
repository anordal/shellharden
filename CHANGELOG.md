# Changelog

This is a log of features and fixes to the source code.
Documentation files are ignored, as are refactoring changes.
For the interested, the commit history is readable too (I know the art of rebasing).

## 4.1.1

* Allow "$*" (no need to rewrite it to "$@" as long as the quotes are on).
* Recognise premature esac to avoid parse error (seen on rustup.sh).
* Write this changelog.

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
