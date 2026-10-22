printf 'a b c\n' | { read x y; echo "$x/$y"; }
printf 'one:two\n' | { IFS=: read p q; echo "$p+$q"; }
echo 'r\ns' | { read -r z; echo "$z"; }
