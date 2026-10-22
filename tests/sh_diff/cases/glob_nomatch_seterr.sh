set -e
for f in /nonexistent/*.txt; do echo "got $f"; done
ls /nonexistent/*.txt 2>/dev/null
echo notreached
