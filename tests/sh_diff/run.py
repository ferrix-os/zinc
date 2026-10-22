#!/usr/bin/env python3
"""Run every case in cases/ under zsh started as `sh` and zinc started as
`sh`, as a script file and as `sh -c`, and report where stdout, stderr or the
exit status differ.

    python3 -I run.py ZINC_BINARY [ZSH_BINARY] [--as=NAME]

Each run gets a fresh $TMPD and a $PATH whose `sh` is the shell under test.
The temporary directory's name is replaced by `$TMPD` before comparing.
"""
import os, subprocess, sys, tempfile, pathlib

here = pathlib.Path(__file__).resolve().parent
args = [a for a in sys.argv[1:] if not a.startswith('-')]
# `--as=zsh` starts both shells under that name instead, to tell what
# differs in sh emulation from what differs in native mode too.
name = next((a.split('=', 1)[1] for a in sys.argv[1:] if a.startswith('--as=')), 'sh')
zinc = os.path.abspath(args[0])
zsh = args[1] if len(args) > 1 else '/usr/bin/zsh'

def run(shell, case, mode):
    with tempfile.TemporaryDirectory(prefix='shdiff-') as tmpd:
        bindir = os.path.join(tmpd, 'bin')
        os.mkdir(bindir)
        sh = os.path.join(bindir, name)
        os.symlink(shell, sh)
        work = os.path.join(tmpd, 'w')
        os.mkdir(work)
        env = {'PATH': bindir + ':/usr/bin:/bin', 'HOME': tmpd, 'TMPD': work,
               'LANG': 'C.UTF-8', 'TERM': 'dumb'}
        text = case.read_text()
        if mode == 'file':
            argv = [sh, str(case)]
        else:
            argv = [sh, '-c', text]
        p = subprocess.run(argv, env=env, cwd=tmpd, capture_output=True,
                           stdin=subprocess.DEVNULL, timeout=20)
        fix = lambda b: b.decode('utf-8', 'replace').replace(work, '$TMPD').replace(tmpd, '$TMP')
        return fix(p.stdout), fix(p.stderr), p.returncode

same = diff = 0
for case in sorted((here / 'cases').glob('*.sh')):
    for mode in ('file', 'c'):
        a = run(zsh, case, mode)
        b = run(zinc, case, mode)
        if a == b:
            same += 1
            continue
        diff += 1
        print(f'DIFF {case.name} ({mode})')
        for label, x, y in (('stdout', a[0], b[0]), ('stderr', a[1], b[1]), ('status', a[2], b[2])):
            if x != y:
                print(f'  {label}:\n    zsh:  {x!r}\n    zinc: {y!r}')
print(f'{same} same, {diff} different')
sys.exit(1 if diff else 0)
