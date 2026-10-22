unset u
echo ${u:-dflt} ${u-dflt2}
e=
echo "[${e:-x}]" "[${e-y}]"
echo ${u:=set}; echo $u
echo ${#u}
p=/a/b/c.tar.gz
echo ${p##*/} ${p%.*} ${p%%.*} ${p#*/}
