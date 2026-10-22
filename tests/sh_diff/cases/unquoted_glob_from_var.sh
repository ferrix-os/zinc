cd "$TMPD"; : > g1.y; : > g2.y
pat='g*.y'
echo $pat
set -- $pat
echo $#
