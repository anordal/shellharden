#!/bin/bash

IFS='' # Nothing to do with IFS

true < "${BASH_SOURCE[0]}"
true <<< fdsaafgaag

echo outside

cat <<-	EOF
	inside
	EOF

echo outside

cat << 'Shit, it'\''s a string'
inside
Shit, it's a string

echo outside

cat << "She said \"a\\b\'c\nd\:e\
f\""
inside
She said "a\b\'c\nd\:ef"

echo outside

cat << 'She said '\"a\\b\'c\nd\:e\
f\"
inside
She said "a\b'cnd:ef"

echo outside

cat << $IFS
inside
$IFS

echo outside

cat << "$IFS"
inside
$IFS

echo outside

cat << "$(eject && printf EOF)"
inside
$(eject && printf EOF)

echo outside
