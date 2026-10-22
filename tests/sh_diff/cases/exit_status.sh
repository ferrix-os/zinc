false; echo $?
(exit 3); echo $?
sh_fn() { return 4; }; sh_fn; echo $?
nosuchcommand_xyz 2>/dev/null; echo $?
