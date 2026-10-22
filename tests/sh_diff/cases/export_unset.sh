export A=1
sh -c 'echo $A' 2>/dev/null || echo noexec
unset A
echo "[${A}]"
