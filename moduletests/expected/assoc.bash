
declare -A assoc
assoc[$1]=$3
assoc[$1]+=_1
assoc[$2]=$3
assoc[$2]+=_2
echo "«${assoc[$1]}»"
echo "«${assoc[$2]}»"
