LOGDIR="$TMPD/plog/dxvk"
mkdir -p "$LOGDIR"
rm -f "$LOGDIR"/*.log
for f in "$LOGDIR"/*.log; do
  [ -e "$f" ] || continue
  echo "found $f"
done
echo done-ok
