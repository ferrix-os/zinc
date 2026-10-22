for v in foo.c bar.h baz; do
case $v in
  *.c|*.h) echo "src $v";;
  *) echo "other $v";;
esac
done
