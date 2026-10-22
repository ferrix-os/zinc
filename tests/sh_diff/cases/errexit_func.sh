set -e
f() { false && true; }
f
echo notreached
