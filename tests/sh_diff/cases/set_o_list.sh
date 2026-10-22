set -o | head -20
set +o | grep -E 'nomatch|shwordsplit|errexit'
