x=global
f() { local x=inner; echo $x; }
f
echo $x
