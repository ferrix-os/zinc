v='a b c'
set -- ${==v}
echo $#
set -- ${=v}
echo $#
p='*.nomatchhere'
echo ${~~p}
echo ${~p}
