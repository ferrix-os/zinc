emulate
emulate zsh -c 'echo inner; emulate; echo /nonexistent/*.q' 2>&1
emulate
echo /nonexistent/*.r
emulate zsh
emulate
v='a b'
for w in $v; do echo "<$w>"; done
emulate sh
for w in $v; do echo "<$w>"; done
