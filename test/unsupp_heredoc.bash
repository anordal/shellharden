#!/bin/bash
echo before
cat - << 'Shit, it'\''s a string'
Only identifiers are supported as heredoc delimiters at this time.
Shit, it's a string
echo after
