echo "$(echo -ne '\n')"
echo "$(echo #`
ls)" && ok
echo "$(echo '`'ls)" && ok
echo "$(echo "$(ls "$oddvar")")"
