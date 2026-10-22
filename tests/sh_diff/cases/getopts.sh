set -- -a -b val rest
while getopts ab: o; do echo "$o ${OPTARG:-}"; done
shift $((OPTIND-1)); echo "$@"
