pat='a*'
case abc in $pat) echo glob-match;; *) echo literal;; esac
case 'a*' in "$pat") echo quoted;; esac
[ abc = $pat ] && echo eq || echo ne
