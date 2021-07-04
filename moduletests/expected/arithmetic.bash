a=3*4
echo $((a)) $((a+2))
((++a))

ver1=(0 9 9)
ver2=(1 0 0)
for ((i = 0; i < ${#ver1[@]}; i++)); do
	if ((10#${ver1[i]} > 10#${ver2[i]})); then
		break
	fi
done
