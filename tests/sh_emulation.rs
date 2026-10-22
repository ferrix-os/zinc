//! zinc started as `sh` behaves as zsh does started as `sh`: in sh
//! emulation, whose option defaults are zsh 5.9's `optns[]` for `OPT_SH`.
//!
//! On Ferrix `/bin/sh` is zinc. A `#!/bin/sh` script -- Steam's
//! compatibility tool, on the N150 box -- ran `rm -f dir/*.log` over a
//! directory with no logs, and zinc stopped it with `no matches found`,
//! which zsh only says as `zsh`: as `sh` an unmatched pattern stays as it is
//! written. Every expected value here is what zsh 5.9 printed, linked as
//! `sh`, on the machine this was written on; `tests/sh_diff/run.py` runs
//! the same comparison against a zsh that is there.

use std::path::PathBuf;
use std::process::Command;

/// zinc linked under `name`, in a directory of this test's own.
fn linked(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("as-{name}"));
    let link = dir.join(name);
    if !link.exists() {
        let _ok = std::fs::create_dir_all(&dir);
        let _ok = std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_zinc"), &link);
    }
    link
}

/// What `name -c script` printed on standard output and standard error, and
/// its status, with zinc started as `name`.
fn run_as(name: &str, args: &[&str], script: &str) -> (String, String, i32) {
    let mut command = Command::new(linked(name));
    command.args(args).arg("-c").arg(script).env_remove("PS4");
    match command.output() {
        Ok(output) => (
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
            output.status.code().unwrap_or(-1),
        ),
        Err(error) => (String::new(), format!("zinc did not start: {error}"), -1),
    }
}

/// Standard output of `sh -c script`.
fn sh(script: &str) -> String {
    run_as("sh", &[], script).0
}

#[test]
fn an_unmatched_pattern_stays_as_written() {
    assert_eq!(
        run_as("sh", &[], "echo /nonexistent/*.log; echo after"),
        ("/nonexistent/*.log\nafter\n".to_owned(), String::new(), 0)
    );
    // The Steam script: `rm -f` over nothing, and on to the next line.
    assert_eq!(
        sh("rm -f /nonexistent/*.log; echo $?; for f in /nonexistent/*.x; do echo \"$f\"; done"),
        "0\n/nonexistent/*.x\n"
    );
    // As `zsh` it is an error, as it is in zsh.
    assert_eq!(
        run_as("zsh", &[], "echo /nonexistent/*.log; echo after"),
        (
            String::new(),
            "zsh:1: no matches found: /nonexistent/*.log\n".to_owned(),
            1
        )
    );
}

#[test]
fn the_name_decides_the_emulation() {
    assert_eq!(sh("emulate"), "sh\n");
    assert_eq!(run_as("ksh", &[], "emulate").0, "ksh\n");
    assert_eq!(run_as("bash", &[], "emulate").0, "sh\n");
    assert_eq!(run_as("zsh", &[], "emulate").0, "zsh\n");
    assert_eq!(run_as("zinc", &[], "emulate").0, "zsh\n");
    assert_eq!(run_as("zsh", &["--emulate", "sh"], "emulate").0, "sh\n");
    // An option on the command line goes on top of the emulation.
    assert_eq!(
        run_as("sh", &["-o", "nomatch"], "echo /nonexistent/*.q").2,
        1
    );
    assert_eq!(
        run_as("sh", &["-x", "--emulate", "sh"], "emulate"),
        (
            String::new(),
            "sh/sh: --emulate: must precede other options\n"
                .replace("sh/sh", &linked("sh").to_string_lossy()),
            1
        )
    );
}

#[test]
fn unquoted_parameters_are_split_and_globbed() {
    assert_eq!(
        sh("v='a b  c'; for w in $v; do echo \"[$w]\"; done"),
        "[a]\n[b]\n[c]\n"
    );
    assert_eq!(
        sh("IFS=:; x='1:2::3'; for w in $x; do echo \"<$w>\"; done"),
        "<1>\n<2>\n<>\n<3>\n"
    );
    assert_eq!(sh("v=' '; set -- $v; echo $#"), "0\n");
    assert_eq!(
        sh("set -- 'a b' c; for a in $*; do echo \"{$a}\"; done"),
        "{a}\n{b}\n{c}\n"
    );
    assert_eq!(sh("v='a b'; set -- ${==v}; echo $#"), "1\n");
    // GLOB_SUBST: what a parameter holds is a pattern, unless quoted.
    assert_eq!(
        sh("p='/*bin'; for f in $p; do [ \"$f\" = /bin ] && echo found; done"),
        "found\n"
    );
    assert_eq!(sh("p='/*bin'; echo \"$p\""), "/*bin\n");
    assert_eq!(
        sh("pat='a*'; case abc in $pat) echo glob;; *) echo literal;; esac"),
        "glob\n"
    );
    // A backslash in the value quotes a wildcard and stays in the word.
    assert_eq!(
        sh("re='foo\\[1\\]'; echo $re; x='a\\b'; echo $x"),
        "foo\\[1\\]\na\\b\n"
    );
}

#[test]
fn sh_options_change_builtins_and_arithmetic() {
    // BSD_ECHO: no escapes without -e.
    assert_eq!(sh("echo 'a\\tb'; echo -e 'c\\td'"), "a\\tb\nc\td\n");
    // OCTAL_ZEROES.
    assert_eq!(sh("echo $(( 010 + 1 )) $((0x10))"), "9 16\n");
    // KSH_ARRAYS: from zero, and the name alone is the first element.
    assert_eq!(
        sh("a=(x y z); echo $a ${a[0]} ${a[1]} ${#a[@]}; a[3]=w; echo ${a[3]}"),
        "x x y 3\nw\n"
    );
    // IGNORE_BRACES.
    assert_eq!(sh("echo {a,b}c x{1..3}"), "{a,b}c x{1..3}\n");
    // NO_FUNCTION_ARGZERO: `$0` in a function is still the shell's.
    assert_eq!(
        sh("f() { echo $0; }; f"),
        format!("{}\n", linked("sh").to_string_lossy())
    );
    // The last element of a pipeline is a child too, as in other sh's.
    assert_eq!(
        sh("n=0; printf '1\\n2\\n' | while read l; do n=$((n+l)); done; echo $n"),
        "0\n"
    );
    assert_eq!(
        run_as(
            "zsh",
            &[],
            "n=0; printf '1\\n2\\n' | while read l; do n=$((n+l)); done; echo $n"
        )
        .0,
        "3\n"
    );
}

#[test]
fn errexit_stops_where_zsh_stops() {
    let e = |s: &str| run_as("sh", &[], &format!("set -e; {s}"));
    assert_eq!(e("false && true; echo A").0, "A\n");
    assert_eq!(
        e("true && false; echo B"),
        (String::new(), String::new(), 1)
    );
    assert_eq!(e("{ false && true; }; echo C").0, "C\n");
    assert_eq!(e("f() { false && true; }; f; echo D").2, 1);
    assert_eq!(e("! true; echo E").0, "E\n");
    assert_eq!(e("if false; then :; fi; echo F").0, "F\n");
    assert_eq!(e("true | false; echo G").2, 1);
    assert_eq!(e("f() { false; echo in; }; f && echo H").0, "in\nH\n");
    assert_eq!(e("for i in 1; do false; done; echo I").0, "");
    assert_eq!(
        e("trap 'echo T' EXIT; false; echo J"),
        ("T\n".to_owned(), String::new(), 1)
    );
}

#[test]
fn nounset_is_an_error_where_a_value_is_wanted() {
    assert_eq!(
        run_as("sh", &[], "set -u; echo ${u:-ok}; echo $u; echo after"),
        (
            "ok\n".to_owned(),
            "zsh:1: u: parameter not set\n".to_owned(),
            1
        )
    );
}

#[test]
fn emulate_sets_options_and_local_ones_come_back() {
    assert_eq!(
        sh("emulate zsh; emulate; v='a b'; for w in $v; do echo \"<$w>\"; done"),
        "zsh\n<a b>\n"
    );
    assert_eq!(
        run_as(
            "sh",
            &[],
            "emulate zsh -c 'emulate; echo /nonexistent/*.q'; emulate; echo /nonexistent/*.r"
        ),
        (
            "zsh\nsh\n/nonexistent/*.r\n".to_owned(),
            "(eval):1: no matches found: /nonexistent/*.q\n".to_owned(),
            0
        )
    );
    // `emulate -L` in a function: its options end with it.
    assert_eq!(
        sh("f() { emulate -L zsh; emulate; }; f; emulate; echo /nonexistent/*.q"),
        "zsh\nsh\n/nonexistent/*.q\n"
    );
    assert_eq!(
        run_as("zsh", &[], "g() { setopt localoptions shwordsplit; }; g; setopt; h() { setopt shwordsplit; }; h; setopt").0,
        "nohashdirs\nnohashdirs\nshwordsplit\n"
    );
}

#[test]
fn options_are_listed_as_zsh_lists_them() {
    assert_eq!(
        sh("echo \"[$-]\"; set -e -u; echo \"[$-]\"; set -f; echo \"[$-]\""),
        "[]\n[eu]\n[efu]\n"
    );
    assert_eq!(run_as("zsh", &[], "echo $-").0, "569X\n");
    assert_eq!(sh("setopt"), "nohashdirs\n");
    assert_eq!(
        sh("set -o | grep -E '^(no)?(nomatch|shwordsplit|aliases) '"),
        "noaliases             off\nnomatch               off\nnoshwordsplit         off\n"
    );
    assert_eq!(
        sh("set +o | grep -E ' (no)?(nomatch|errexit)$'"),
        "set +o errexit\nset +o nomatch\n"
    );
    assert_eq!(
        run_as("zsh", &[], "setopt nosuchthing; echo $?"),
        (
            "1\n".to_owned(),
            "zsh:setopt:1: no such option: nosuchthing\n".to_owned(),
            0
        )
    );
}

/// What zsh does in either emulation, which the same run found zinc not
/// doing.
#[test]
fn zsh_rules_in_either_emulation() {
    // A program whose pattern matched nothing fails alone; zsh globs after
    // it has forked. A builtin's stops the script.
    assert_eq!(
        run_as("zsh", &[], "ls /nonexistent/*.l; echo after $?; echo /nonexistent/*.l; echo never"),
        (
            "after 1\n".to_owned(),
            "zsh:1: no matches found: /nonexistent/*.l\nzsh:1: no matches found: /nonexistent/*.l\n".to_owned(),
            1
        )
    );
    // An error inside `eval` stops the `eval`, `${u:?}` the shell.
    assert_eq!(
        run_as("zsh", &[], "eval 'echo /x/*; echo in'; echo after $?").0,
        "after 1\n"
    );
    assert_eq!(
        run_as("zsh", &[], "eval 'echo ${u:?gone}'; echo after").2,
        1
    );
    // An array joined into one word is joined with the first of `$IFS`.
    // (`$w` unquoted is split at that comma again, sh emulation being on.)
    assert_eq!(
        sh("set -- a b c; IFS=,; echo \"$*\"; w=$*; echo \"$w\" $w; unset IFS; echo \"$*\""),
        "a,b,c\na,b,c a b c\na b c\n"
    );
    // `=cmd` is the program's path.
    assert_eq!(
        run_as("zsh", &[], "echo =sh =nonexistent-cmd").1,
        "zsh:1: nonexistent-cmd not found\n"
    );
    // A file runs a line at a time: what an earlier line did is in force,
    // and a parse error stops it only where it is.
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let script = dir.join("by-line.sh");
    let _ok = std::fs::write(&script, "echo a\necho b\n( echo c\n");
    let output = Command::new(linked("sh")).arg(&script).output();
    assert_eq!(
        output
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default(),
        "a\nb\n"
    );
}

#[test]
fn xtrace_shows_the_commands() {
    assert_eq!(
        run_as(
            "sh",
            &[],
            "set -x; echo 'a b' c \"it's\" ''; x=1; for i in 1; do :; done"
        )
        .1,
        "+ echo 'a b' c 'it'\\''s' ''\n+ x=1 \n+ i=1\n+ :\n"
    );
    assert_eq!(run_as("zsh", &[], "set -x; echo hi").1, "+zsh:1> echo hi\n");
}
