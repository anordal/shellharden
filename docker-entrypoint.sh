#!/bin/sh
set -eu
# run command if it is not starting with a "-" and is an executable in PATH
if [ "${#}" -gt 0 ] \
    && [ "${1#-}" = "${1}" ] \
    && command -v "${1}" > "/dev/null" 2>&1; then
    exec "${@}"
else
    exec /bin/shfmt "${@}" # else default to run the command
fi
exit 0
