case $- in *f*) echo f;; esac
set -o | grep -E '^(nomatch|shwordsplit|globsubst|bsdecho|banghist|ksharrays|posixbuiltins|shglob|nullglob|octalzeroes|ignorebraces|shoptionletters|unset|equals|functionargzero|promptpercent) ' 
