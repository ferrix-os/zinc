set -e
false && true
echo after-andand
{ false || true; }
if false; then echo no; fi
f() { false; echo in-f; }
f && echo f-ok
! true
echo before
true | false
echo notreached
