#!/bin/bash

abc=$'a\nb\nc'

usage_heredoc(){
cat <<	EOF
$abc
EOF
}

usage_heretic(){
	echo "$abc"
}

usage_hermetic(){
	echo "$abc"
}

usage_heredoc
usage_heretic
usage_hermetic
