//! zinc: a zsh-compatible shell.
//!
//! The acceptance criterion is that oh-my-zsh runs in it. Behaviour is
//! measured against zsh 5.9, whose source is the reference for every rule a
//! module here ports; each module names the zsh file it follows.

#![allow(
    dead_code,
    reason = "the shell is being built bottom-up; stages land before their callers"
)]

mod arith;
mod ast;
mod builtins;
mod cond;
mod dquote;
mod exec;
mod expand;
mod input;
mod jobs;
mod lex;
mod lexword;
mod options;
mod param;
mod parse;
mod parsectl;
mod pattern;
mod prompt;
mod qual;
mod regex;
mod shell;
mod tok;
mod wcwidth;
mod zformat;
mod zle;
mod zstyle;

use std::os::unix::ffi::OsStrExt;

use lex::Lexer;
use shell::{Flow, Shell};

/// `zinc --tokens FILE`: one token per line, for comparing with zsh's lexer.
fn dump_tokens(sh: &Shell, text: &[u8]) {
    let mut lx = Lexer::new(tok::metafy(text), sh.lex_opts());
    let mut outb = Vec::new();
    loop {
        lx.zshlex(sh);
        outb.extend_from_slice(format!("{:?}", lx.tok).as_bytes());
        if let Some(s) = &lx.tokstr {
            let mut shown = s.clone();
            tok::untokenize(&mut shown);
            outb.push(b' ');
            outb.extend(tok::unmetafy(&shown));
        }
        outb.push(b'\n');
        if matches!(lx.tok, lex::Tok::Endinput | lex::Tok::Lexerr) {
            break;
        }
    }
    let _ok = exec::write_fd(1, &outb);
}

/// `zinc --ast FILE`: parse and print each event's tree.
fn dump_ast(sh: &Shell, text: &[u8]) -> i32 {
    let mut lx = Lexer::new(tok::metafy(text), sh.lex_opts());
    loop {
        let mut p = parse::Parser::new(&mut lx, sh);
        match p.parse_event() {
            Ok(Some(list)) => {
                let _ok = exec::write_fd(1, format!("{list:#?}\n").as_bytes());
            }
            Ok(None) => return 0,
            Err(e) => {
                let _ok = exec::write_fd(1, format!("zinc:{}: {}\n", e.lineno, e.msg).as_bytes());
                return 1;
            }
        }
    }
}

/// Leave the shell: the EXIT trap, then the process.
fn finish(sh: &mut Shell) -> ! {
    if let Some(trap) = sh.exit_trap.take() {
        let status = sh.status;
        sh.flow = Flow::Normal;
        exec::run_string(sh, &trap);
        sh.status = status;
    }
    exec::exit_now(sh.status)
}

/// True if `text` ends inside an unfinished construct and needs more lines.
fn incomplete(sh: &Shell, text: &[u8]) -> bool {
    let mut lx = Lexer::new(text.to_vec(), sh.lex_opts());
    let r = {
        let mut p = parse::Parser::new(&mut lx, sh);
        p.parse_all()
    };
    r.is_err() && lx.input.rest().is_empty() && lx.input.stop
}

/// Read one line from fd 0, unbuffered so programs see the rest of the input.
fn read_line() -> Option<Vec<u8>> {
    let mut line = Vec::new();
    let mut b = [0u8; 1];
    loop {
        // SAFETY: b is one writable byte.
        let n = unsafe { libc::read(0, b.as_mut_ptr().cast(), 1) };
        if n < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        if n <= 0 {
            return if line.is_empty() { None } else { Some(line) };
        }
        line.push(b[0]);
        if b[0] == b'\n' {
            return Some(line);
        }
    }
}

fn interactive_loop(sh: &mut Shell) -> ! {
    let mut buffer: Vec<u8> = Vec::new();
    let mut editor = zle::Editor::default();
    sh.at_prompt = true;
    // Whether the last thing typed was an `exit` that was refused because
    // jobs were suspended. zsh refuses once and obeys the second time, so
    // that a session is never ended by a keystroke while work is stopped in
    // it, and never held hostage either.
    let mut warned = false;
    loop {
        let which: &[u8] = if buffer.is_empty() { b"PS1" } else { b"PS2" };
        let ps = sh.get(which).map(|v| v.joined()).unwrap_or_default();
        let prompt = prompt::expand(sh, &ps);
        let Some(line) = editor.read_line(sh, &prompt) else {
            if !buffer.is_empty() {
                exec::run_string(sh, &buffer);
            }
            let _ok = exec::write_fd(2, b"\n");
            finish(sh);
        };
        buffer.extend(tok::metafy(&line));
        if incomplete(sh, &buffer) {
            continue;
        }
        let text = std::mem::take(&mut buffer);
        editor.add_history(&tok::unmetafy(&text));
        // What a job started on this line is called in `jobs`: a pipeline's
        // elements are expanded in the processes that run them, so the line
        // is the only place the whole command is known at once.
        sh.line_text = trimmed(&tok::unmetafy(&text));
        exec::run_string(sh, &text);
        sh.line_text.clear();
        sh.at_prompt = true;
        reap(sh);
        if sh.flow == Flow::Exit {
            if leaving(sh, &mut warned) {
                finish(sh);
            }
            sh.flow = Flow::Normal;
            continue;
        }
        sh.flow = Flow::Normal;
        warned = false;
    }
}

/// Whether the shell should leave now, or refuse this once because jobs are
/// suspended in it and say so.
fn leaving(sh: &mut Shell, warned: &mut bool) -> bool {
    if *warned || !sh.jobs.any_stopped() {
        return true;
    }
    *warned = true;
    let message = format!("{}: you have suspended jobs.\n", sh.name);
    let _ok = exec::write_fd(2, message.as_bytes());
    false
}

/// `text` without the white space around it: a job's line, as typed.
fn trimmed(text: &[u8]) -> Vec<u8> {
    let start = text.iter().position(|b| !b.is_ascii_whitespace());
    let end = text.iter().rposition(|b| !b.is_ascii_whitespace());
    match (start, end) {
        (Some(start), Some(end)) => text.get(start..=end).unwrap_or(&[]).to_vec(),
        _ => Vec::new(),
    }
}

/// Report what the jobs did while the user was typing, and forget the ones
/// that have finished: the prompt is where zsh says so without `NOTIFY`.
fn reap(sh: &mut Shell) {
    sh.jobs.notify();
}

fn source_if_exists(sh: &mut Shell, path: &[u8]) {
    if let Ok(text) = std::fs::read(String::from_utf8_lossy(path).as_ref()) {
        exec::run_file(sh, &tok::metafy(&text));
        if sh.flow == Flow::Exit {
            finish(sh);
        }
        sh.flow = Flow::Normal;
    }
}

/// The name zsh goes by, which decides its emulation: the last part of
/// `argv[0]` without a leading `-`, and for `su`, which starts the shell
/// under its own name, the shell `$SHELL` names (zsh's `zsh_main`).
fn zsh_name(argv0: &[u8]) -> Vec<u8> {
    let base = |path: &[u8]| -> Vec<u8> {
        let last = path.rsplit(|&c| c == b'/').next().unwrap_or(b"zinc");
        last.strip_prefix(b"-").unwrap_or(last).to_vec()
    };
    let name = base(argv0);
    if name == b"su"
        && let Some(shell) = std::env::var_os("SHELL")
    {
        let shell = shell.as_bytes();
        if !shell.is_empty() && shell != argv0 {
            return base(shell);
        }
    }
    name
}

/// What the command line asked for besides options.
#[derive(Debug)]
struct Invocation {
    /// `-c`: the first operand is the command.
    command: bool,
    /// Where the operands begin in `argv`.
    operands: usize,
}

/// Read the options on the command line into `sh`, as zsh's `parseopts`
/// does at the top level. The emulation is installed before the first
/// option is set -- by `--emulate`, or else by the shell's name -- so that
/// `sh -o nomatch` ends with the option on. `Err(None)` is a clean exit
/// (`--version`), `Err(Some(message))` a refusal.
fn parse_options(
    sh: &mut Shell,
    args: &[Vec<u8>],
    name: &[u8],
) -> Result<Invocation, Option<String>> {
    let mut emulation: Option<Vec<u8>> = Some(name.to_vec());
    let install = |sh: &mut Shell, emulation: &mut Option<Vec<u8>>| {
        if let Some(name) = emulation.take() {
            sh.emulate(&name, true);
        }
    };
    let mut command = false;
    let mut k = 1;
    'args: while let Some(a) = args.get(k) {
        let on = match a.first() {
            Some(b'-') => true,
            Some(b'+') => false,
            _ => break,
        };
        k += 1;
        // A lone `-` ends the options, as `--` does.
        if a.len() == 1 || a.as_slice() == b"--" {
            break;
        }
        if on && a.get(1) == Some(&b'-') {
            let long = a.get(2..).unwrap_or(&[]);
            match long {
                b"version" => {
                    let _ok = exec::write_fd(1, b"zsh 5.9 (zinc)\n");
                    return Err(None);
                }
                b"tokens" | b"ast" => {
                    let file = args.get(k).cloned().unwrap_or_default();
                    let Ok(text) = std::fs::read(String::from_utf8_lossy(&file).as_ref()) else {
                        return Err(Some("cannot read file".to_owned()));
                    };
                    if long == b"tokens" {
                        dump_tokens(sh, &text);
                        return Err(None);
                    }
                    exec::exit_now(dump_ast(sh, &text));
                }
                b"emulate" => {
                    let Some(mode) = args.get(k) else {
                        return Err(Some("--emulate: argument required".to_owned()));
                    };
                    if emulation.is_none() {
                        return Err(Some("--emulate: must precede other options".to_owned()));
                    }
                    emulation = Some(mode.clone());
                    k += 1;
                }
                _ => {
                    install(sh, &mut emulation);
                    let option: Vec<u8> = long
                        .iter()
                        .map(|&c| if c == b'-' { b'_' } else { c })
                        .collect();
                    if !sh.set_option(&option, true) {
                        return Err(Some(format!(
                            "no such option: {}",
                            String::from_utf8_lossy(&option)
                        )));
                    }
                }
            }
            continue;
        }
        let mut letters = a.iter().skip(1);
        while let Some(&c) = letters.next() {
            // `-b` ends the options after this argument -- unless the sh
            // letters are in use, where it is `notify`. Before any option
            // has installed the emulation, they are not, which is zsh's
            // order too.
            let sh_letters = emulation.is_none() && sh.opt("shoptionletters");
            if c == b'b' && !sh_letters {
                install(sh, &mut emulation);
                for &c in letters.by_ref() {
                    set_letter(sh, c, on)?;
                }
                break 'args;
            }
            install(sh, &mut emulation);
            match c {
                b'c' => command = true,
                b'o' => {
                    let rest: Vec<u8> = letters.by_ref().copied().collect();
                    let option = if rest.is_empty() {
                        let Some(next) = args.get(k) else {
                            return Err(Some("string expected after -o".to_owned()));
                        };
                        k += 1;
                        next.clone()
                    } else {
                        rest
                    };
                    if !sh.set_option(&option, on) {
                        return Err(Some(format!(
                            "no such option: {}",
                            String::from_utf8_lossy(&option)
                        )));
                    }
                }
                _ => set_letter(sh, c, on)?,
            }
        }
    }
    install(sh, &mut emulation);
    Ok(Invocation {
        command,
        operands: k,
    })
}

/// One option letter on the command line.
fn set_letter(sh: &mut Shell, c: u8, on: bool) -> Result<(), Option<String>> {
    match options::letter(c, sh.opt("shoptionletters")) {
        Some((option, invert)) => {
            let _ok = sh.set_option(option.as_bytes(), on != invert);
            Ok(())
        }
        None => Err(Some(format!("bad option: -{}", char::from(c)))),
    }
}

fn main() {
    // Rust starts programs with SIGPIPE ignored, which every child would
    // inherit; a shell's children expect the default.
    // SAFETY: SIG_DFL is a valid disposition for SIGPIPE.
    let _old = unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
    let args: Vec<Vec<u8>> = std::env::args_os().map(|a| a.as_bytes().to_vec()).collect();
    let argv0 = args.first().cloned().unwrap_or_else(|| b"zinc".to_vec());
    let base = zsh_name(&argv0);
    // Messages name the shell `zsh` whatever it was started as, as zsh's
    // do once it is running; only the ones about its own command line name
    // it as it was run.
    let mut sh = Shell::new("zsh".to_owned());
    let argv0_shown = String::from_utf8_lossy(&argv0).into_owned();
    if base != b"zinc" {
        sh.set_scalar(b"ZSH_NAME", tok::metafy(&base));
    }
    sh.argzero = tok::metafy(&argv0);
    // A login shell: `-l`, or a name beginning with `-`, which is how
    // `login` and `su -` start one.
    let login_name = argv0
        .rsplit(|&c| c == b'/')
        .next()
        .is_some_and(|name| name.first() == Some(&b'-'));
    let opts = match parse_options(&mut sh, &args, &base) {
        Ok(opts) => opts,
        Err(Some(message)) => {
            let _ok = exec::write_fd(2, format!("{argv0_shown}: {message}\n").as_bytes());
            exec::exit_now(1);
        }
        Err(None) => exec::exit_now(0),
    };
    // PS4 as the emulation has it: zsh's names the script and line, the
    // Bourne shells' is a plus.
    if std::env::var_os("PS4").is_none() {
        let ps4: &[u8] = if sh.emulation.bourne() {
            b"+ "
        } else {
            b"+%N:%i> "
        };
        sh.set_scalar(b"PS4", ps4.to_vec());
    }
    let k = opts.operands;
    let run_command = opts.command;
    let force_interactive = sh.opt("interactive");
    let login = login_name || sh.opt("login");
    let _old = sh.options_mut().insert("login".to_owned(), login);
    let no_rcs = !sh.opt("rcs");
    // zsh hashes directories by default only in an interactive shell.
    let _old = sh
        .options_mut()
        .insert("hashdirs".to_owned(), force_interactive);
    let mut rest: Vec<Vec<u8>> = args
        .get(k..)
        .unwrap_or(&[])
        .iter()
        .map(|a| tok::metafy(a))
        .collect();
    if run_command && rest.is_empty() {
        let _ok = exec::write_fd(
            2,
            format!("{}: string expected after -c\n", sh.name).as_bytes(),
        );
        exec::exit_now(1);
    }
    let command = run_command.then(|| rest.remove(0));

    if let Some(cmd) = command {
        if let Some((zero, pos)) = rest.split_first() {
            sh.argzero = zero.clone();
            sh.positional = pos.to_vec();
        }
        exec::run_string(&mut sh, &cmd);
        finish(&mut sh);
    }
    // `-s`: the commands come from standard input whatever the operands
    // are, which become the positional parameters.
    let from_stdin = sh.opt("shinstdin");
    if from_stdin {
        sh.positional = std::mem::take(&mut rest);
    }
    if let Some((file, pos)) = rest.split_first() {
        sh.argzero = file.clone();
        sh.positional = pos.to_vec();
        let Ok(text) = std::fs::read(String::from_utf8_lossy(&tok::unmetafy(file)).as_ref()) else {
            let message = format!(
                "{argv0_shown}: can't open input file: {}\n",
                String::from_utf8_lossy(&tok::unmetafy(file))
            );
            let _ok = exec::write_fd(2, message.as_bytes());
            exec::exit_now(127);
        };
        sh.script = file.clone();
        exec::run_file(&mut sh, &tok::metafy(&text));
        finish(&mut sh);
    }
    // SAFETY: isatty has no memory-safety preconditions.
    let tty = unsafe { libc::isatty(0) } == 1;
    sh.interactive = force_interactive || tty;
    // What `$-` and `setopt` show of how the shell was started.
    for (option, on) in [
        ("interactive", sh.interactive),
        ("monitor", sh.interactive),
        ("shinstdin", true),
        ("hashdirs", sh.interactive),
    ] {
        let _old = sh.options_mut().insert(option.to_owned(), on);
    }
    if !sh.interactive {
        let mut text = Vec::new();
        while let Some(line) = read_line() {
            text.extend(line);
        }
        exec::run_file(&mut sh, &tok::metafy(&text));
        finish(&mut sh);
    }
    // Job control, before the signals below are ignored: taking the terminal
    // means stopping the shell's own group until it has it, and a shell that
    // ignored SIGTTIN could not be stopped that way. A shell whose terminal
    // cannot be had -- no controlling terminal to be given -- runs on without
    // it: jobs are still tracked and waited for, but nothing is moved between
    // process groups.
    let _control = sh.jobs.take_terminal();
    for sig in [
        libc::SIGINT,
        libc::SIGQUIT,
        libc::SIGTERM,
        libc::SIGTSTP,
        libc::SIGTTIN,
        libc::SIGTTOU,
    ] {
        // SAFETY: SIG_IGN is a valid disposition for these signals.
        let _old = unsafe { libc::signal(sig, libc::SIG_IGN) };
    }
    if sh.emulation.bourne() {
        // As sh or ksh, zsh reads what those shells read: the profiles for
        // a login shell, and `$ENV` for an interactive one.
        if login {
            source_if_exists(&mut sh, b"/etc/profile");
            if let Some(home) = sh.get(b"HOME").map(|v| tok::unmetafy(&v.joined())) {
                source_if_exists(&mut sh, &[home.as_slice(), b"/.profile"].concat());
            }
        }
        if let Some(env) = sh.get(b"ENV").map(|v| tok::unmetafy(&v.joined()))
            && !env.is_empty()
        {
            source_if_exists(&mut sh, &env);
        }
    } else if !no_rcs {
        let dir = sh
            .get(b"ZDOTDIR")
            .or_else(|| sh.get(b"HOME"))
            .map(|v| v.joined())
            .unwrap_or_default();
        // zsh's order, each global file before the user's own, and the
        // profile and login files only for a login shell.
        let mut files: Vec<(&[u8], &[u8])> = vec![(b"/etc/zshenv", b"/.zshenv")];
        if login {
            files.push((b"/etc/zprofile", b"/.zprofile"));
        }
        files.push((b"/etc/zshrc", b"/.zshrc"));
        if login {
            files.push((b"/etc/zlogin", b"/.zlogin"));
        }
        for (global, own) in files {
            source_if_exists(&mut sh, global);
            let path = [dir.as_slice(), own].concat();
            source_if_exists(&mut sh, &tok::unmetafy(&path));
        }
    }
    interactive_loop(&mut sh);
}
