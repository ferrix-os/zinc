v=val
cat <<END
v is $v
$(echo sub)
END
cat <<'END'
v is $v
END
cat <<-END
	tabbed $v
	END
