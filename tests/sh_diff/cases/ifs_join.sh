set -- a b c
IFS=,
echo "$*"
v="x,y,,z"
for w in $v; do echo "[$w]"; done
unset IFS
echo "$*"
