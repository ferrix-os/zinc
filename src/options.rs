//! The option table and emulation: which options exist, what each one is
//! called by its aliases and letters, and which are on in each of the shells
//! zsh can pretend to be (zsh's `options.c`).
//!
//! zsh started as `sh` or `ksh` -- the name it was run by, after any leading
//! `-` -- or with `--emulate`, begins in that shell's emulation, and the
//! `emulate` builtin changes it later. An emulation is nothing but a set of
//! option defaults, so the tables below, copied from zsh 5.9's `optns[]`, are
//! the whole of what it means.

use crate::shell::Table;

/// The shell being emulated (zsh's `EMULATE_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Emulation {
    Csh,
    Ksh,
    Sh,
    Zsh,
}

impl Emulation {
    /// The emulation a name asks for, by zsh's rule in `emulate()`: its
    /// first letter, after an `r` for a restricted shell. `sh` and `bash`
    /// are sh, `ksh` ksh, `csh` csh, anything else zsh.
    pub(crate) fn from_name(name: &[u8]) -> Emulation {
        let first = match name {
            [b'r', second, ..] => *second,
            [first, ..] => *first,
            [] => b'z',
        };
        match first {
            b'c' => Emulation::Csh,
            b'k' => Emulation::Ksh,
            b's' | b'b' => Emulation::Sh,
            _ => Emulation::Zsh,
        }
    }

    /// What `emulate` with no arguments prints.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Emulation::Csh => "csh",
            Emulation::Ksh => "ksh",
            Emulation::Sh => "sh",
            Emulation::Zsh => "zsh",
        }
    }

    fn bit(self) -> u8 {
        match self {
            Emulation::Csh => CSH,
            Emulation::Ksh => KSH,
            Emulation::Sh => SH,
            Emulation::Zsh => ZSH,
        }
    }

    /// Whether this is one of the Bourne shells, whose option letters
    /// `SH_OPTION_LETTERS` follows.
    pub(crate) fn bourne(self) -> bool {
        matches!(self, Emulation::Sh | Emulation::Ksh)
    }
}

const CSH: u8 = 1;
const KSH: u8 = 2;
const SH: u8 = 4;
const ZSH: u8 = 8;
const ALL: u8 = CSH | KSH | SH | ZSH;
const BOURNE: u8 = KSH | SH;
const BSHELL: u8 = KSH | SH | ZSH;
const NONBOURNE: u8 = ALL & !BOURNE;
const NONZSH: u8 = ALL & !ZSH;
/// Set by `emulate` even when it is not asked to reset everything.
const EMULATE: u8 = 16;
/// Never set by an emulation: whether the shell is interactive and the like.
const SPECIAL: u8 = 32;

/// Every option, with the emulations it is on in by default.
pub(crate) const OPTIONS: &[(&str, u8)] = &[
    ("aliases", EMULATE | ALL),
    ("aliasfuncdef", EMULATE | BOURNE),
    ("allexport", EMULATE),
    ("alwayslastprompt", ALL),
    ("alwaystoend", 0),
    ("appendcreate", EMULATE | BOURNE),
    ("appendhistory", ALL),
    ("autocd", EMULATE),
    ("autocontinue", 0),
    ("autolist", ALL),
    ("automenu", ALL),
    ("autonamedirs", 0),
    ("autoparamkeys", ALL),
    ("autoparamslash", ALL),
    ("autopushd", 0),
    ("autoremoveslash", ALL),
    ("autoresume", 0),
    ("badpattern", EMULATE | NONBOURNE),
    ("banghist", NONBOURNE),
    ("bareglobqual", EMULATE | ZSH),
    ("bashautolist", 0),
    ("bashrematch", 0),
    ("beep", ALL),
    ("bgnice", EMULATE | NONBOURNE),
    ("braceccl", EMULATE),
    ("bsdecho", EMULATE | SH),
    ("caseglob", ALL),
    ("casematch", ALL),
    ("casepaths", 0),
    ("cbases", 0),
    ("cprecedences", EMULATE | NONZSH),
    ("cdablevars", EMULATE),
    ("cdsilent", 0),
    ("chasedots", EMULATE),
    ("chaselinks", EMULATE),
    ("checkjobs", EMULATE | ZSH),
    ("checkrunningjobs", EMULATE | ZSH),
    ("clobber", EMULATE | ALL),
    ("clobberempty", 0),
    ("combiningchars", 0),
    ("completealiases", 0),
    ("completeinword", 0),
    ("continueonerror", 0),
    ("correct", 0),
    ("correctall", 0),
    ("cshjunkiehistory", EMULATE | CSH),
    ("cshjunkieloops", EMULATE | CSH),
    ("cshjunkiequotes", EMULATE | CSH),
    ("cshnullcmd", EMULATE | CSH),
    ("cshnullglob", EMULATE | CSH),
    ("debugbeforecmd", ALL),
    ("emacs", 0),
    ("equals", EMULATE | ZSH),
    ("errexit", EMULATE),
    ("errreturn", EMULATE),
    ("exec", ALL),
    ("extendedglob", EMULATE),
    ("extendedhistory", CSH),
    ("evallineno", EMULATE | ZSH),
    ("flowcontrol", ALL),
    ("forcefloat", 0),
    ("functionargzero", EMULATE | NONBOURNE),
    ("glob", EMULATE | ALL),
    ("globalexport", EMULATE | ZSH),
    ("globalrcs", ALL),
    ("globassign", EMULATE | CSH),
    ("globcomplete", 0),
    ("globdots", EMULATE),
    ("globstarshort", EMULATE),
    ("globsubst", EMULATE | NONZSH),
    ("hashcmds", ALL),
    ("hashdirs", ALL),
    ("hashexecutablesonly", 0),
    ("hashlistall", ALL),
    ("histallowclobber", 0),
    ("histbeep", ALL),
    ("histexpiredupsfirst", 0),
    ("histfcntllock", 0),
    ("histfindnodups", 0),
    ("histignorealldups", 0),
    ("histignoredups", 0),
    ("histignorespace", 0),
    ("histlexwords", 0),
    ("histnofunctions", 0),
    ("histnostore", 0),
    ("histsubstpattern", EMULATE),
    ("histreduceblanks", 0),
    ("histsavebycopy", ALL),
    ("histsavenodups", 0),
    ("histverify", 0),
    ("hup", EMULATE | ZSH),
    ("ignorebraces", EMULATE | SH),
    ("ignoreclosebraces", EMULATE),
    ("ignoreeof", 0),
    ("incappendhistory", 0),
    ("incappendhistorytime", 0),
    ("interactive", SPECIAL),
    ("interactivecomments", BOURNE),
    ("ksharrays", EMULATE | BOURNE),
    ("kshautoload", EMULATE | BOURNE),
    ("kshglob", EMULATE | KSH),
    ("kshoptionprint", EMULATE | KSH),
    ("kshtypeset", 0),
    ("kshzerosubscript", 0),
    ("listambiguous", ALL),
    ("listbeep", ALL),
    ("listpacked", 0),
    ("listrowsfirst", 0),
    ("listtypes", ALL),
    ("localoptions", EMULATE | KSH),
    ("localloops", EMULATE),
    ("localpatterns", EMULATE),
    ("localtraps", EMULATE | KSH),
    ("login", SPECIAL),
    ("longlistjobs", 0),
    ("magicequalsubst", EMULATE),
    ("mailwarning", 0),
    ("markdirs", 0),
    ("menucomplete", 0),
    ("monitor", SPECIAL),
    ("multibyte", ALL),
    ("multifuncdef", EMULATE | ZSH),
    ("multios", EMULATE | ZSH),
    ("nomatch", EMULATE | NONBOURNE),
    ("notify", ZSH),
    ("nullglob", EMULATE),
    ("numericglobsort", EMULATE),
    ("octalzeroes", EMULATE | SH),
    ("overstrike", 0),
    ("pathdirs", EMULATE),
    ("pathscript", EMULATE | BOURNE),
    ("pipefail", EMULATE),
    ("posixaliases", EMULATE | BOURNE),
    ("posixargzero", EMULATE),
    ("posixbuiltins", EMULATE | BOURNE),
    ("posixcd", EMULATE | BOURNE),
    ("posixidentifiers", EMULATE | BOURNE),
    ("posixjobs", EMULATE | BOURNE),
    ("posixstrings", EMULATE | BOURNE),
    ("posixtraps", EMULATE | BOURNE),
    ("printeightbit", 0),
    ("printexitvalue", 0),
    ("privileged", SPECIAL),
    ("promptbang", KSH),
    ("promptcr", ALL),
    ("promptpercent", NONBOURNE),
    ("promptsp", ALL),
    ("promptsubst", BOURNE),
    ("pushdignoredups", EMULATE),
    ("pushdminus", EMULATE),
    ("pushdsilent", 0),
    ("pushdtohome", EMULATE),
    ("rcexpandparam", EMULATE),
    ("rcquotes", EMULATE),
    ("rcs", ALL),
    ("recexact", 0),
    ("rematchpcre", 0),
    ("restricted", SPECIAL),
    ("rmstarsilent", BOURNE),
    ("rmstarwait", 0),
    ("sharehistory", KSH),
    ("shfileexpansion", EMULATE | BOURNE),
    ("shglob", EMULATE | BOURNE),
    ("shinstdin", SPECIAL),
    ("shnullcmd", EMULATE | BOURNE),
    ("shoptionletters", EMULATE | BOURNE),
    ("shortloops", EMULATE | NONBOURNE),
    ("shortrepeat", EMULATE),
    ("shwordsplit", EMULATE | BOURNE),
    ("singlecommand", SPECIAL),
    ("singlelinezle", KSH),
    ("sourcetrace", 0),
    ("sunkeyboardhack", 0),
    ("transientrprompt", 0),
    ("trapsasync", 0),
    ("typesetsilent", EMULATE | BOURNE),
    ("typesettounset", EMULATE | BOURNE),
    ("unset", EMULATE | BSHELL),
    ("verbose", 0),
    ("vi", 0),
    ("warncreateglobal", EMULATE),
    ("warnnestedvar", EMULATE),
    ("xtrace", 0),
    ("zle", SPECIAL),
    ("dvorak", 0),
];
/// Other shells' names for options, and whether each is the opposite.
const ALIASES: &[(&str, &str, bool)] = &[
    ("braceexpand", "ignorebraces", true),
    ("dotglob", "globdots", false),
    ("hashall", "hashcmds", false),
    ("histappend", "appendhistory", false),
    ("histexpand", "banghist", false),
    ("log", "histnofunctions", true),
    ("mailwarn", "mailwarning", false),
    ("onecmd", "singlecommand", false),
    ("physical", "chaselinks", false),
    ("promptvars", "promptsubst", false),
    ("stdin", "shinstdin", false),
    ("trackall", "hashcmds", false),
];
/// `zsh -x`, `set -x`: the option each letter stands for, and whether it
/// turns it off. zsh's `zshletters`.
const ZSH_LETTERS: &[(u8, &str, bool)] = &[
    (b'0', "correct", false),
    (b'1', "printexitvalue", false),
    (b'2', "badpattern", true),
    (b'3', "nomatch", true),
    (b'4', "globdots", false),
    (b'5', "notify", false),
    (b'6', "bgnice", false),
    (b'7', "ignoreeof", false),
    (b'8', "markdirs", false),
    (b'9', "autolist", false),
    (b'B', "beep", true),
    (b'C', "clobber", true),
    (b'D', "pushdtohome", false),
    (b'E', "pushdsilent", false),
    (b'F', "glob", true),
    (b'G', "nullglob", false),
    (b'H', "rmstarsilent", false),
    (b'I', "ignorebraces", false),
    (b'J', "autocd", false),
    (b'K', "banghist", true),
    (b'L', "sunkeyboardhack", false),
    (b'M', "singlelinezle", false),
    (b'N', "autopushd", false),
    (b'O', "correctall", false),
    (b'P', "rcexpandparam", false),
    (b'Q', "pathdirs", false),
    (b'R', "longlistjobs", false),
    (b'S', "recexact", false),
    (b'T', "cdablevars", false),
    (b'U', "mailwarning", false),
    (b'V', "promptcr", true),
    (b'W', "autoresume", false),
    (b'X', "listtypes", false),
    (b'Y', "menucomplete", false),
    (b'Z', "zle", false),
    (b'a', "allexport", false),
    (b'd', "globalrcs", true),
    (b'e', "errexit", false),
    (b'f', "rcs", true),
    (b'g', "histignorespace", false),
    (b'h', "histignoredups", false),
    (b'i', "interactive", false),
    (b'k', "interactivecomments", false),
    (b'l', "login", false),
    (b'm', "monitor", false),
    (b'n', "exec", true),
    (b'p', "privileged", false),
    (b'r', "restricted", false),
    (b's', "shinstdin", false),
    (b't', "singlecommand", false),
    (b'u', "unset", true),
    (b'v', "verbose", false),
    (b'w', "chaselinks", false),
    (b'x', "xtrace", false),
    (b'y', "shwordsplit", false),
];
/// The letters under `SH_OPTION_LETTERS` (zsh's `kshletters`).
const KSH_LETTERS: &[(u8, &str, bool)] = &[
    (b'C', "clobber", true),
    (b'T', "trapsasync", false),
    (b'X', "markdirs", false),
    (b'a', "allexport", false),
    (b'b', "notify", false),
    (b'e', "errexit", false),
    (b'f', "glob", true),
    (b'i', "interactive", false),
    (b'l', "login", false),
    (b'm', "monitor", false),
    (b'n', "exec", true),
    (b'p', "privileged", false),
    (b'r', "restricted", false),
    (b's', "shinstdin", false),
    (b't', "singlecommand", false),
    (b'u', "unset", true),
    (b'v', "verbose", false),
    (b'x', "xtrace", false),
];

/// The option `name` stands for, by zsh's `optlookup`: underscores and case
/// do not count, an alias names its option, and a `no` in front that is not
/// part of a name inverts it. The flag says whether it was inverted.
pub(crate) fn lookup(name: &[u8]) -> Option<(&'static str, bool)> {
    let key: String = name
        .iter()
        .filter(|&&c| c != b'_')
        .map(|&c| char::from(c.to_ascii_lowercase()))
        .collect();
    let exact = |key: &str| -> Option<(&'static str, bool)> {
        if let Some((n, _)) = OPTIONS.iter().find(|(n, _)| *n == key) {
            return Some((n, false));
        }
        ALIASES
            .iter()
            .find(|(n, _, _)| *n == key)
            .map(|(_, target, invert)| (*target, *invert))
    };
    if let Some(found) = exact(&key) {
        return Some(found);
    }
    let rest = key.strip_prefix("no")?;
    exact(rest).map(|(n, invert)| (n, !invert))
}

/// Whether `name` is on by default in `emulation` (zsh's `defset`).
pub(crate) fn default_in(name: &str, emulation: Emulation) -> bool {
    OPTIONS
        .iter()
        .find(|(n, _)| *n == name)
        .is_some_and(|(_, flags)| flags & emulation.bit() != 0)
}

/// Put `options` into `emulation`, as zsh's `installemulation` does: every
/// option that belongs to an emulation, or with `fully` every option that is
/// not special, takes the emulation's default.
pub(crate) fn install(options: &mut Table<String, bool>, emulation: Emulation, fully: bool) {
    for (name, flags) in OPTIONS {
        if (fully && flags & SPECIAL == 0) || flags & EMULATE != 0 {
            let _old = options.insert((*name).to_owned(), flags & emulation.bit() != 0);
        }
    }
}

/// The option a letter stands for, and whether the letter turns it off, by
/// the zsh letters or, under `SH_OPTION_LETTERS`, the sh ones.
pub(crate) fn letter(c: u8, sh_letters: bool) -> Option<(&'static str, bool)> {
    let table = if sh_letters { KSH_LETTERS } else { ZSH_LETTERS };
    table
        .iter()
        .find(|(l, _, _)| *l == c)
        .map(|(_, name, invert)| (*name, *invert))
}

/// `$-`: the letter of every option whose letter is in effect, in the order
/// of the letter table.
pub(crate) fn dash(options: &Table<String, bool>, sh_letters: bool) -> Vec<u8> {
    let table = if sh_letters { KSH_LETTERS } else { ZSH_LETTERS };
    table
        .iter()
        .filter(|(_, name, invert)| options.get(*name).copied().unwrap_or(false) != *invert)
        .map(|(l, _, _)| *l)
        .collect()
}

/// Every option's name, sorted, as zsh's listings walk its table.
pub(crate) fn names() -> Vec<&'static str> {
    let mut all: Vec<&'static str> = OPTIONS.iter().map(|(n, _)| *n).collect();
    all.sort_unstable();
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_picks_its_emulation_by_the_first_letter() {
        assert_eq!(Emulation::from_name(b"sh"), Emulation::Sh);
        assert_eq!(Emulation::from_name(b"bash"), Emulation::Sh);
        assert_eq!(Emulation::from_name(b"rsh"), Emulation::Sh);
        assert_eq!(Emulation::from_name(b"ksh"), Emulation::Ksh);
        assert_eq!(Emulation::from_name(b"csh"), Emulation::Csh);
        assert_eq!(Emulation::from_name(b"zsh"), Emulation::Zsh);
        assert_eq!(Emulation::from_name(b"zinc"), Emulation::Zsh);
    }

    #[test]
    fn names_and_aliases_are_looked_up_as_zsh_does() {
        assert_eq!(lookup(b"NO_MATCH"), Some(("nomatch", false)));
        assert_eq!(lookup(b"nonomatch"), Some(("nomatch", true)));
        assert_eq!(lookup(b"notify"), Some(("notify", false)));
        assert_eq!(lookup(b"SH_WORD_SPLIT"), Some(("shwordsplit", false)));
        assert_eq!(lookup(b"braceexpand"), Some(("ignorebraces", true)));
        assert_eq!(lookup(b"nobraceexpand"), Some(("ignorebraces", false)));
        assert_eq!(lookup(b"nosuchoption"), None);
    }

    #[test]
    fn sh_emulation_turns_off_nomatch_and_on_word_splitting() {
        let mut o = Table::default();
        install(&mut o, Emulation::Sh, true);
        for on in [
            "shwordsplit",
            "globsubst",
            "bsdecho",
            "ksharrays",
            "octalzeroes",
            "shglob",
        ] {
            assert_eq!(o.get(on), Some(&true), "{on}");
        }
        for off in [
            "nomatch",
            "banghist",
            "equals",
            "functionargzero",
            "bareglobqual",
        ] {
            assert_eq!(o.get(off), Some(&false), "{off}");
        }
        install(&mut o, Emulation::Zsh, false);
        assert_eq!(o.get("nomatch"), Some(&true));
        // Not an emulation option: only a full emulation resets it.
        assert_eq!(o.get("banghist"), Some(&false));
    }
}
