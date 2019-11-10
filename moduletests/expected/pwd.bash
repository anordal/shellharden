# implemented
echo "$PWD."
echo "$PWD."
echo "${PWD}a"
echo "${PWD}a"

echo "$PWD."
echo "$PWD."
echo "${PWD}a"
echo "${PWD}a"

echo "$1$PWD."
echo "$1$PWD."
echo "$1${PWD}a"
echo "$1${PWD}a"

# not optimally handled
echo "$PWD".
echo "$PWD".
echo "${PWD}"a
echo "${PWD}"a

echo "$1$PWD".
echo "$1$PWD".
echo "$1${PWD}"a
echo "$1${PWD}"a

# not implemented
echo "$( pwd)"
echo "$(pwd )"
