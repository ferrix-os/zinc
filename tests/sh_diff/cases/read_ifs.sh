printf 'a:b:c\n' | { IFS=: read x y; echo "$x|$y"; }
printf '  lead trail  \n' | { read l; echo "[$l]"; }
printf 'one two\n' > "$TMPD/f"
read a b < "$TMPD/f"; echo "$b $a"
