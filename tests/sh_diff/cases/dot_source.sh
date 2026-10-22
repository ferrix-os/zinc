cd "$TMPD"
echo 'echo sourced; rm -f /nonexistent/*.log; echo inside-after' > lib.sh
. ./lib.sh
echo after
