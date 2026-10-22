files=$(echo /nonexistent/*.log)
echo "$files"
set -- /nonexistent/*.log
echo $# "$1"
