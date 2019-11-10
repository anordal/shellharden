# implemented
echo "$(pwd)."
echo "`pwd`."
echo "$(pwd)a"
echo "`pwd`a"

echo $(pwd)"."
echo `pwd`"."
echo $(pwd)"a"
echo `pwd`"a"

echo $1$(pwd)"."
echo $1`pwd`"."
echo $1$(pwd)"a"
echo $1`pwd`"a"

# not optimally handled
echo $(pwd).
echo `pwd`.
echo $(pwd)a
echo `pwd`a

echo $1$(pwd).
echo $1`pwd`.
echo $1$(pwd)a
echo $1`pwd`a

# not implemented
echo "$( pwd)"
echo "$(pwd )"
