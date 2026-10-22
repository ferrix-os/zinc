f() { emulate -L zsh; echo /nonexistent/*.q 2>/dev/null || echo failed; emulate; }
f
emulate
echo /nonexistent/*.q
g() { setopt localoptions nonomatch; setopt shwordsplit; }
setopt nomatch
g
setopt | grep -E 'nomatch|shwordsplit'
