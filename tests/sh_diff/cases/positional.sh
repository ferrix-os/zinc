set -- "a b" c
for a in "$@"; do echo "<$a>"; done
for a in $*; do echo "{$a}"; done
echo "$*"
echo $#
