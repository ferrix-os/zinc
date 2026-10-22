x=5
echo $((x+=2)) $x
unset x
echo ${x:-gone}
readonly r=1
echo $r
f() { echo "args:$#"; }
f "" ""
command -v ls >/dev/null && echo has-ls
type cd | head -1
