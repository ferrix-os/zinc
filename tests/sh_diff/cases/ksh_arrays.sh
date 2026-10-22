a=(zero one two)
echo ${a[0]} ${a[2]} ${#a[@]} ${a[@]}
a[3]=three
echo ${a[3]} $((a[1]))
echo ${#a}
