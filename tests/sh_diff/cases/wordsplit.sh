v='a b  c'
for w in $v; do echo "[$w]"; done
set -- $v
echo $#
IFS=:
x='1:2::3'
for w in $x; do echo "<$w>"; done
