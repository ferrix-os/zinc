//! printf's escapes and conversions, byte for byte.
//!
//! POSIX.1-2024 printf: the format operand takes the escapes of XBD's File
//! Format Notation (`\\ \a \b \f \n \r \t \v`) and `\ddd`, one to three
//! octal digits; a `%b` argument takes the same letters and `\0ddd`, zero
//! to three digits after the `0`, and its `\c` ends all of printf's output.
//! Where POSIX leaves a case open the expected value is what zsh 5.9 prints,
//! checked against bash and dash for the cases all three share. They are
//! written out rather than measured, so the tests say the same thing on a
//! machine with no zsh.
//!
//! Compared as bytes: a NUL an escape makes must reach the output, which a
//! C-string or UTF-8 view of it would hide.

use std::process::Command;

/// What `zinc -c script` wrote to standard output, and its exit status.
fn run(script: &str) -> (Vec<u8>, i32) {
    match Command::new(env!("CARGO_BIN_EXE_zinc"))
        .args(["-c", script])
        .output()
    {
        Ok(output) => (output.stdout, output.status.code().unwrap_or(-1)),
        Err(error) => (format!("zinc did not start: {error}").into_bytes(), -1),
    }
}

fn out(script: &str) -> Vec<u8> {
    run(script).0
}

/// The two commands that found the bug, on the N150 over SSH (2026-10-17):
/// `\0003` is `\000` and then a `3`, and three-digit octal is a byte.
#[test]
fn the_two_commands_that_found_the_bug() {
    assert_eq!(
        out(r#"printf "+\0003\000-\0000\000""#),
        b"+\x003\x00-\x000\x00"
    );
    assert_eq!(
        out(r#"printf "\053\000\063\000\055\000\060\000""#),
        b"+\x003\x00-\x000\x00"
    );
    // The same through a pipe, as dd saw it.
    assert_eq!(
        out(r#"printf "+\0003\000-\0000\000" | cat"#),
        b"+\x003\x00-\x000\x00"
    );
}

/// Every escape of the format operand, one at a time.
#[test]
fn each_format_escape() {
    for (esc, byte) in [
        (r"\\", b'\\'),
        (r"\a", 7),
        (r"\b", 8),
        (r"\f", 12),
        (r"\n", b'\n'),
        (r"\r", b'\r'),
        (r"\t", b'\t'),
        (r"\v", 11),
        (r"\e", 27),
        (r#"\""#, b'"'),
    ] {
        assert_eq!(
            out(&format!("printf '<{esc}>'")),
            [b'<', byte, b'>'],
            "{esc}"
        );
    }
}

/// `\ddd`: one, two or three octal digits, and no more; `\0` is NUL.
#[test]
fn octal_in_the_format_is_one_to_three_digits() {
    assert_eq!(out(r"printf '\0'"), b"\0");
    assert_eq!(out(r"printf '\1|\12|\101|\1012'"), b"\x01|\n|A|A2");
    assert_eq!(out(r"printf '\0101'"), b"\x081");
    assert_eq!(out(r"printf '\18|\08'"), b"\x018|\x008");
    // Above a byte wraps, as zsh's does.
    assert_eq!(out(r"printf '\777|\400'"), b"\xff|\x00");
    // A NUL in the middle does not end the output.
    assert_eq!(out(r"printf 'a\0b%sc\n' X"), b"a\0bXc\n");
    assert_eq!(out(r"printf '\000\000\000'"), b"\0\0\0");
}

/// What is not an escape stays as it was; zsh's `\x`, `\u` and `\c`.
#[test]
fn other_backslashes_in_the_format() {
    assert_eq!(out(r"printf '\q|\8|\'\''|\'"), br"\q|\8|\'|\");
    assert_eq!(out(r"printf '\x41|\x4a|\x|\xg'"), b"A|J|\0|\0g");
    assert_eq!(out(r"printf '\u00e9'"), "\u{e9}".as_bytes());
    assert_eq!(out(r"printf 'a\cb'; printf c"), b"ac");
    assert_eq!(out("printf '100%%\\n'"), b"100%\n");
}

/// `%b`: `\0ddd` with up to three digits after the `0`, and `\ddd` is not
/// an escape there.
#[test]
fn b_takes_zero_and_up_to_three_digits() {
    assert_eq!(
        out(r"printf '%b|' '\0' '\00011' '\0101' '\01011'"),
        b"\0|\x011|A|A1|"
    );
    assert_eq!(
        out(r"printf '%b|' '\101' '\8' '\q' '\x41'"),
        br"\101|\8|\q|A|"
    );
    assert_eq!(out(r"printf '%b' 'a\nb\tc\\d\n'"), b"a\nb\tc\\d\n");
    assert_eq!(out(r"printf '%b|' '\0777' '\0400'"), b"\xff|\x00|");
    // `%s` leaves the same argument alone.
    assert_eq!(out(r"printf '%s|%b|' 'x\ny' 'x\ny'"), b"x\\ny|x\ny|");
    assert_eq!(out(r"printf '%5b|' '\t'"), b"    \t|");
}

/// `\c` in a `%b` argument ends printf's output there: the rest of the
/// format, and the arguments it would have been used again for.
#[test]
fn backslash_c_in_b_ends_everything() {
    assert_eq!(out(r"printf 'x%bz\n' 'a\cb'; echo after"), b"xaafter\n");
    assert_eq!(out(r"printf '%b|' '\0' '\c1' after"), b"\0|");
}

/// More arguments than the format takes: the format is used again, and an
/// argument it runs short of is empty or zero.
#[test]
fn the_format_is_reused() {
    assert_eq!(out(r"printf '%s-%s\n' 1 2 3"), b"1-2\n3-\n");
    assert_eq!(out(r"printf '%s %d\n' a 1 b 2 c"), b"a 1\nb 2\nc 0\n");
    assert_eq!(out(r"printf '<\0>' x y"), b"<\0>");
    assert_eq!(out(r"printf 'abc\n' extra"), b"abc\n");
    assert_eq!(out(r"printf '[%s]\n'"), b"[]\n");
}

/// The integer conversions, with C's flags and precision.
#[test]
fn integer_conversions() {
    assert_eq!(
        out(r"printf '%d %i %o %x %X %u %c|\n' 42 -3 8 255 255 7 hello"),
        b"42 -3 10 ff FF 7 h|\n"
    );
    assert_eq!(
        out(r"printf '%5d|%-5d|%05d|%+d|% d|%#x|%#o|%#X\n' 1 2 42 5 5 255 8 255"),
        b"    1|2    |00042|+5| 5|0xff|010|0XFF\n"
    );
    assert_eq!(
        out(r"printf '%.3d|%8.3d|%-8.3x|%.0d|%#.0o|%+05d|%05s\n' 5 5 255 0 0 3 ab"),
        b"005|     005|0ff     ||0|+0003|   ab\n"
    );
    assert_eq!(
        out(r"printf '%u|%x\n' -1 -1"),
        b"18446744073709551615|ffffffffffffffff\n"
    );
    assert_eq!(
        out(r"printf '%d|%d|%d|%d\n' \'A 0x10 1e3 ''"),
        b"65|16|1000|0\n"
    );
    assert_eq!(
        out(r"printf '%*d|%-*d|%*d|\n' 5 1 4 2 -3 7"),
        b"    1|2   |7  |\n"
    );
}

/// The floating conversions, as C prints them.
#[test]
fn float_conversions() {
    assert_eq!(
        out(r"printf '%5.2f|%e|%g|%f\n' 3.14159 31415.9 0.0001 1+1"),
        b" 3.14|3.141590e+04|0.0001|2.000000\n"
    );
    assert_eq!(
        out(
            r"printf '%.0f|%#.0f|%G|%E|%g|%g|%g|%#g|%g\n' 2.5 2 1e-10 12345 100000 1000000 123.456 1 0"
        ),
        b"2|2.|1E-10|1.234500E+04|100000|1e+06|123.456|1.00000|0\n"
    );
    assert_eq!(
        out(
            r"printf '%+.3e|%010.3f|%-10.2e|% f|%+g|%.*f\n' 1234.5 -3.14159 0.000123 1 -0.5 -1 2.5"
        ),
        b"+1.234e+03|-00003.142|1.23e-04  | 1.000000|-0.5|2.500000\n"
    );
    assert_eq!(out(r"printf '%f %e %G\n' inf -inf nan"), b"inf -inf NAN\n");
}

/// Strings: width counts characters, precision cuts.
#[test]
fn string_conversions() {
    assert_eq!(
        out(r"printf '%5s|%-5s|%.2s|%10.4s|%5c|%-3c|\n' ab ab abc abcdefg a b"),
        b"   ab|ab   |ab|      abcd|    a|b  |\n"
    );
    assert_eq!(out("printf '%3s|' \u{e9}"), "  \u{e9}|".as_bytes());
}

/// A directive printf does not know is an error that ends the output, as
/// zsh has it.
#[test]
fn an_unknown_directive_is_an_error() {
    assert_eq!(run(r"printf 'a%kb\n' x"), (b"a".to_vec(), 1));
    assert_eq!(run(r"printf '%d\n' 3x").1, 1);
}

/// echo and print share the escapes, each with zsh's octal rule.
#[test]
fn echo_and_print_escapes() {
    assert_eq!(
        out(r"echo 'a\0003b|\0101|\101|\8|\x41'"),
        b"a\x03b|A|\\101|\\8|A\n"
    );
    assert_eq!(out(r"echo 'a\cb'; echo c"), b"ac\n");
    assert_eq!(out(r"echo -E '\101'; echo -e '\0101'"), b"\\101\nA\n");
    assert_eq!(out(r"print '\101|\0101|\1|\q|\0'"), b"A|\x081|\x01|q|\0\n");
}
