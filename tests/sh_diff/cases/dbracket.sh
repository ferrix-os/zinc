x=abc
if [[ $x == a* ]]; then echo match; fi
[[ -n $x && $x != b ]] && echo both
[[ abc =~ ^a(b)c$ ]] && echo re
