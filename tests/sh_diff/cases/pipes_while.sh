n=0
printf '1\n2\n3\n' | while read l; do n=$((n+l)); done
echo $n
