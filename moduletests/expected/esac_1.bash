"$(case "$PATH" in
	*something*)
		# Backticks must be escaped in backtick context, but these aren't.
		echo "$(echo 'I can see it in your eyes')"
	;;
esac)"
