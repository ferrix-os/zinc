cd "$TMPD"
for f in *.nomatch; do echo "f=$f"; done
[ -e *.nomatch ] || echo absent
