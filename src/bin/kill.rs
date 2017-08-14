#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;
#[macro_use]
extern crate lazy_static;
#[cfg(target_os = "redox")]
extern crate syscall;
#[cfg(all(unix, not(target_os = "redox")))]
extern crate nix;

use extra::option::OptionalExt;
use std::io::{stdout, stderr, Write};
use std::fmt;
use std::collections::HashMap;

const MAN_PAGE: &'static str = /* @MANSTART{kill} */ r#"
NAME
    kill - send a signal

SYNOPSIS
    kill [-s signal_name] pid ...
    kill -signal_name pid ...
    kill -signal_number pid ...
    kill -l
    kill -l signal_name ...

DESCRIPTION
    The kill utility sends a signal to the processes specified by the pid
    operands.

    Only the super-user may send signals to other users' processes.

OPTIONS
    --help, -h
        print this message

    -s  signal_name
        A symbolic signal name specifying the signal to be sent instead
        of the default TERM.

    -signal_name
        A symbolic signal name specifying the signal to be sent instead
        of the default TERM.

    -signal_number
        A non-negative decimal integer, specifying the signal to be sent
        instead of the default TERM.

    -l [signal_name1 [signal_name2]...]
        If no operand is given, list the signal names; otherwise, converts
        the given signal number to a name,
        or the given signal name to a number.
"#; /* @MANEND */
// -------------------------------------

type SigT = i32;
#[allow(unused)]
static SIG_TYPE_MAX: SigT = std::i32::MAX;
static DEFAULT_SIG_NUM: SigT = 15;

#[derive(Debug, PartialEq)]
enum CliParseErr {
    WrongSignalNum(u64),
    InvalidSigSpec(String),
    DuplicatedSig((SigT, String)),
    WrongPid(String),
    ArgExpected(String),
    NoPids,
}

impl fmt::Display for CliParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliParseErr::WrongSignalNum(sig_num) => {
                if sig_num >= 128 {
                    write!(f, "Wrong signal number: {}, signal should be less than 128", sig_num)
                } else {
                    write!(f, "Wrong signal number: {}", sig_num)
                }
            }
            CliParseErr::InvalidSigSpec(ref sig_str) => write!(f, "{}: invalid signal specification", sig_str),
            CliParseErr::DuplicatedSig((fst, ref snd_str)) => write!(f, "Duplicated signal specification: {} and {}", fst, snd_str),
            CliParseErr::WrongPid(ref pid) => write!(f, "Wrong PID: {}", pid),
            CliParseErr::ArgExpected(ref opt) => write!(f, "Option '{}' requires an argument", opt),
            CliParseErr::NoPids => write!(f, "No one PID was specified"),
        }
    }
}

#[derive(Debug, PartialEq)]
struct ActKill {
    signal: SigT,
    pids: Vec<usize>,
}

#[derive(Debug, PartialEq)]
enum Action {
    Kill(ActKill),
    Help,
    PrintAllSignals,
    PrintSignalsInfo(Vec<String>),
}
// -------------------------------------

lazy_static! {
    static ref SIGNAL_NAME_TO_NUM_MAP: HashMap<&'static str, SigT> = {
        [("HUP", 1),
         ("INT", 2),
         ("QUIT", 3),
         ("ILL", 4),
         ("TRAP", 5),
         ("ABRT", 6),
         ("FPE", 8),
         ("KILL", 9),
         ("USR1", 10),
         ("SEGV", 11),
         ("USR2", 12),
         ("PIPE", 13),
         ("ALRM", 14),
         ("TERM", 15),
         ("TKFLT", 16),
         ("CHLD", 17),
         ("CONT", 18),
         ("STOP", 19),
         ("TSTP", 20),
         ("TTIN", 21),
         ("TTOU", 22),
        ].iter()
         .cloned()
         .collect()
    };

    static ref SIGNAL_NUM_TO_NAME_MAP: HashMap<SigT, &'static str> = {
        SIGNAL_NAME_TO_NUM_MAP
            .clone()
            .into_iter()
            .map(|(name, num)| (num, name))
            .collect()
    };
}

fn sig_list_to_table<'a, I>(elems: I, cols: usize) -> String
    where I: IntoIterator<Item = (&'a str, SigT)>
{
    use std::cmp;
    use std::fmt::Write as WriteFmt;

    let mut max_char_cnt = 0;
    let mut str_list: Vec<String> = Vec::new();
    for (name, id) in elems.into_iter() {
        let mut el = String::new();
        write!(&mut el, "{: >id_width$}) SIG{}", id, name, id_width = 2).unwrap();
        max_char_cnt = cmp::max(el.chars().count(), max_char_cnt);
        str_list.push(el);
    }

    let mut res = String::new();
    for (i, el) in str_list.iter().enumerate() {
        if i > 0 {
            let is_first_col = i % cols == 0;
            let delimiter = if is_first_col { '\n' } else { ' ' };
            res.push(delimiter);
        }

        let is_last_col = (i + 1) % cols == 0;
        if !is_last_col {
            write!(&mut res, "{: <width$}", el, width = max_char_cnt).unwrap();
        } else {
            res.push_str(el);
        }
    }

    res
}

#[cfg(target_os = "redox")]
fn validate_signal_num(sig: u64) -> Result<SigT, CliParseErr> {
    if sig > 0x7F {
        Err(CliParseErr::WrongSignalNum(sig))
    } else {
        Ok(sig as SigT)
    }
}
#[cfg(all(unix, not(target_os = "redox")))]
fn validate_signal_num(sig: u64) -> Result<SigT, CliParseErr> {
    if sig > SIG_TYPE_MAX as u64 {
        Err(CliParseErr::WrongSignalNum(sig))
    } else {
        match nix::sys::signal::Signal::from_c_int(sig as SigT) {
            Ok(_) => Ok(sig as SigT),
            Err(_) => Err(CliParseErr::WrongSignalNum(sig)),
        }
    }
}
#[cfg(all(not(unix), not(target_os = "redox")))]
fn validate_signal_num(sig: u64) -> Result<SigT, CliParseErr> {
    Ok(sig as SigT)
}

fn parse_signal_name(sig_name: &str) -> Result<SigT, CliParseErr> {
    let orig_sig_name = sig_name;

    let sig_name = sig_name.to_uppercase();
    let sig_name = if sig_name.starts_with("SIG") {
        &sig_name[3..]
    } else {
        &sig_name
    };

    if let Some(sig_num) = SIGNAL_NAME_TO_NUM_MAP.get(sig_name) {
        Ok(*sig_num)
    } else {
        Err(CliParseErr::InvalidSigSpec(orig_sig_name.to_owned()))
    }
}

fn parse_signal(argv: &str, prev_sig: Option<SigT>) -> Result<SigT, CliParseErr> {
    if let Some(prev_sig_num) = prev_sig {
        Err(CliParseErr::DuplicatedSig((prev_sig_num, argv.to_owned())))

    } else if let Ok(sig_num) = argv.parse::<u64>() {
        validate_signal_num(sig_num)

    } else {
        parse_signal_name(argv)
    }
}

fn parse_opt_arg<A: Iterator<Item = String>>(args: &mut A, opt_name: &str) -> Result<String, CliParseErr> {
    args.next()
        .ok_or(CliParseErr::ArgExpected(opt_name.to_owned()))
}

fn parse_optional_signal_list<A: Iterator<Item = String>>(args: &mut A) -> Result<Vec<String>, CliParseErr> {
    let mut res = Vec::new();

    while let Some(argv) = args.next() {
        res.push(argv.to_owned());
    }
    Ok(res)
}

fn parse_cli<A: Iterator<Item = String>>(args: A) -> Result<Action, CliParseErr> {

    let mut signal: Option<SigT> = None;
    let mut pids: Vec<usize> = Vec::new();

    let mut args = args.skip(1);

    while let Some(arg) = args.next() {
        if arg == "-s" {
            let argv = try!(parse_opt_arg(&mut args, &arg));
            signal = Some(try!(parse_signal(&argv, signal)));

        } else if arg == "-h" || arg == "--help" {
            return Ok(Action::Help);

        } else if arg == "-l" {
            let sig_list = try!(parse_optional_signal_list(&mut args));
            if sig_list.is_empty() {
                return Ok(Action::PrintAllSignals);
            } else {
                return Ok(Action::PrintSignalsInfo(sig_list));
            }

        } else if arg.starts_with("-") {
            signal = Some(try!(parse_signal(&arg[1..], signal)));

        } else {
            if let Ok(pid) = arg.parse::<usize>() {
                pids.push(pid);
            } else {
                return Err(CliParseErr::WrongPid(arg));
            }
        }
    }

    if pids.is_empty() {
        return Err(CliParseErr::NoPids);
    }

    Ok(Action::Kill(ActKill {
                        signal: signal.unwrap_or(DEFAULT_SIG_NUM),
                        pids: pids,
                    }))
}

#[cfg(target_os = "redox")]
fn do_kill(kill_info: &ActKill) {
    let mut stderr = stderr();

    for pid in &kill_info.pids {
        syscall::kill(*pid, kill_info.signal as usize).unwrap_or_else(|e| {
                                                                          writeln!(&mut stderr, "{}. PID:{}", e, pid).try(&mut stderr);
                                                                          0
                                                                      });
    }
}
#[cfg(all(unix, not(target_os = "redox")))]
fn do_kill(kill_info: &ActKill) {
    use nix::sys::signal as nix_signal;

    let mut stderr = stderr();

    let sig = nix_signal::Signal::from_c_int(kill_info.signal).try(&mut stderr);

    for pid in &kill_info.pids {
        nix_signal::kill(*pid as SigT, sig).unwrap_or_else(|e| writeln!(&mut stderr, "{}. PID:{}", e, pid).try(&mut stderr));
    }
}
#[cfg(all(not(unix), not(target_os = "redox")))]
fn do_kill(kill_info: &ActKill) {
    use extra::io::fail;
    fail(&format!("Non UNIX systems are not supported."), &mut stderr());
}

fn print_help() {
    let mut stderr = stderr();
    let stdout = stdout();
    let mut stdout = stdout.lock();

    stdout
        .write_all(MAN_PAGE.trim_left().as_bytes())
        .try(&mut stderr);
    stdout.flush().try(&mut stderr);
}

fn print_all_signals() {
    let mut sig_list = SIGNAL_NAME_TO_NUM_MAP
        .clone()
        .into_iter()
        .collect::<Vec<_>>();
    sig_list.sort_by_key(|&(_, id)| id);

    let output = sig_list_to_table(sig_list, 3);
    writeln!(&mut stdout(), "{}", output).try(&mut stderr());
}

fn get_signal_info(sig_specification: &str) -> Result<String, CliParseErr> {
    if let Ok(sig_num) = sig_specification.parse::<u64>() {
        let sig_num = try!(validate_signal_num(sig_num));
        match SIGNAL_NUM_TO_NAME_MAP.get(&sig_num) {
            Some(sig_name) => Ok((*sig_name).to_owned()),
            None => Err(CliParseErr::InvalidSigSpec(sig_specification.to_owned())),
        }
    } else {
        let sig_num = try!(parse_signal_name(sig_specification));
        Ok(sig_num.to_string())
    }
}

fn print_signals_info(sig_list: &Vec<String>) {
    let mut stdout = stdout();
    let mut stderr = stderr();

    for sig_specification in sig_list {
        match get_signal_info(sig_specification) {
            Ok(sig_info) => writeln!(&mut stdout, "{}", sig_info).try(&mut stderr),
            Err(err) => writeln!(&mut stderr, "{}", err).try(&mut stderr),
        }
    }
}

fn main() {
    use extra::io::fail;

    use std::env;

    if cfg!(all(not(unix), not(target_os = "redox"))) {
        fail(&format!("Non UNIX systems are not supported."), &mut stderr());
    }

    match parse_cli(env::args()) {
        Ok(ref action) => match *action {
            Action::Kill(ref kill_info) => do_kill(kill_info),
            Action::Help => print_help(),
            Action::PrintAllSignals => print_all_signals(),
            Action::PrintSignalsInfo(ref sig_list) => print_signals_info(&sig_list),
        },
        Err(err) => fail(&format!("Wrong arguments: {}. Use --help to see the usage.", err),
                         &mut stderr()),
    };
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! string_vec [
        ($($e:expr),*) => ({
            let mut _temp = ::std::vec::Vec::new();
            $(_temp.push($e.to_owned());)*
            _temp
        })
    ];

    #[test]
    fn test_parse_signal_name() {
        assert_eq!(parse_signal_name("TERM"), Ok(15));
        assert_eq!(parse_signal_name("HUP"), Ok(1));
        assert_eq!(parse_signal_name("term"), Ok(15));
        assert_eq!(parse_signal_name("SIGTERM"), Ok(15));
        assert_eq!(parse_signal_name("SIGterm"), Ok(15));
        assert_eq!(parse_signal_name("sigTERM"), Ok(15));
        assert_eq!(parse_signal_name("WRONG"), Err(CliParseErr::InvalidSigSpec("WRONG".to_owned())));
        assert_eq!(parse_signal_name("SIGWRONG"),
                   Err(CliParseErr::InvalidSigSpec("SIGWRONG".to_owned())));
    }

    #[test]
    fn test_parse_default_sig() {
        {
            let args: Vec<String> = string_vec!["kill", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 15,
                                               pids: vec![12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }

        {
            let args: Vec<String> = string_vec!["kill", "9", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 15,
                                               pids: vec![9, 12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }
    }

    #[test]
    fn test_parse_cli_num_sig_via_dash() {
        {
            let args: Vec<String> = string_vec!["kill", "-9", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 9,
                                               pids: vec![12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }

        {
            let args: Vec<String> = string_vec!["kill", "-2", "9", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 2,
                                               pids: vec![9, 12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }
    }

    #[test]
    fn test_parse_cli_named_sig_via_dash() {
        {
            let args: Vec<String> = string_vec!["kill", "-KILL", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 9,
                                               pids: vec![12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }

        {
            let args: Vec<String> = string_vec!["kill", "-SIGINT", "9", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 2,
                                               pids: vec![9, 12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }
    }

    #[test]
    fn test_parse_cli_sig_via_dash_s() {
        {
            let args: Vec<String> = string_vec!["kill", "-s", "KILL", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 9,
                                               pids: vec![12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }

        {
            let args: Vec<String> = string_vec!["kill", "-s", "SIGINT", "9", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 2,
                                               pids: vec![9, 12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }

        {
            let args: Vec<String> = string_vec!["kill", "-s", "4", "9", "12"];
            let expected = Ok(Action::Kill(ActKill {
                                               signal: 4,
                                               pids: vec![9, 12],
                                           }));
            assert_eq!(parse_cli(args.into_iter()), expected);
        }
    }

    #[test]
    fn test_parse_cli_dash_l() {
        let args: Vec<String> = string_vec!["kill", "-l"];
        let expected = Ok(Action::PrintAllSignals);
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-l", "9"];
        let expected = Ok(Action::PrintSignalsInfo(string_vec!["9"]));
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-l", "TER1"];
        let expected = Ok(Action::PrintSignalsInfo(string_vec!["TER1"]));
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-l", "-9"];
        let expected = Ok(Action::PrintSignalsInfo(string_vec!["-9"]));
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-l", "QUIT", "TERM"];
        let expected = Ok(Action::PrintSignalsInfo(string_vec!["QUIT", "TERM"]));
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-l", "9", "TERM", "29", "TER1"];
        let expected = Ok(Action::PrintSignalsInfo(string_vec!["9", "TERM", "29", "TER1"]));
        assert_eq!(parse_cli(args.into_iter()), expected);
    }

    #[test]
    fn test_to_table() {
        let args = vec![("KILL", 9), ("TERM", 15)];
        let expected = " 9) SIGKILL\n15) SIGTERM".to_owned();
        assert_eq!(sig_list_to_table(args, 1), expected);

        let args = vec![("HUP", 1), ("INT", 2), ("KILL", 9), ("TERM", 15)];
        let expected = " 1) SIGHUP\n 2) SIGINT\n 9) SIGKILL\n15) SIGTERM".to_owned();
        assert_eq!(sig_list_to_table(args, 1), expected);

        let args = vec![("HUP", 1), ("INT", 2), ("KILL", 9), ("TERM", 15)];
        let expected = " 1) SIGHUP   2) SIGINT\n 9) SIGKILL 15) SIGTERM".to_owned();
        assert_eq!(sig_list_to_table(args, 2), expected);

        let args = vec![("HUP", 1), ("INT", 2), ("KILL", 9), ("TERM", 15)];
        let expected = " 1) SIGHUP   2) SIGINT   9) SIGKILL\n15) SIGTERM".to_owned();
        assert_eq!(sig_list_to_table(args, 3), expected);

        let args = vec![("HUP", 1), ("INT", 2), ("KILL", 9), ("TERM", 15)];
        let expected = " 1) SIGHUP   2) SIGINT   9) SIGKILL 15) SIGTERM".to_owned();
        assert_eq!(sig_list_to_table(args, 4), expected);
    }

    #[test]
    fn test_get_signal_info() {
        assert_eq!(get_signal_info("int"), Ok("2".to_owned()));
        assert_eq!(get_signal_info("KILL"), Ok("9".to_owned()));
        assert_eq!(get_signal_info("sigKILL"), Ok("9".to_owned()));
        assert_eq!(get_signal_info("SigKILl"), Ok("9".to_owned()));

        assert_eq!(get_signal_info("2"), Ok("INT".to_owned()));
        assert_eq!(get_signal_info("1"), Ok("HUP".to_owned()));
        assert_eq!(get_signal_info("15"), Ok("TERM".to_owned()));

        assert_eq!(get_signal_info("int1"), Err(CliParseErr::InvalidSigSpec("int1".to_owned())));
        assert_eq!(get_signal_info("-9"), Err(CliParseErr::InvalidSigSpec("-9".to_owned())));
        assert_eq!(get_signal_info("1189"), Err(CliParseErr::WrongSignalNum(1189)));
    }

    #[test]
    fn test_parse_cli_err_duplicated_sig() {
        let args: Vec<String> = string_vec!["kill", "-s", "KILL", "-3", "12"];
        let expected = Err(CliParseErr::DuplicatedSig((9, "3".to_owned())));
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-3", "-s", "KILL", "12"];
        let expected = Err(CliParseErr::DuplicatedSig((3, "KILL".to_owned())));
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-3", "-KILL", "12"];
        let expected = Err(CliParseErr::DuplicatedSig((3, "KILL".to_owned())));
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-3", "-9", "12"];
        let expected = Err(CliParseErr::DuplicatedSig((3, "9".to_owned())));
        assert_eq!(parse_cli(args.into_iter()), expected);
    }

    #[test]
    fn test_parse_cli_err_no_pids() {
        let args: Vec<String> = string_vec!["kill", "-9"];
        let expected = Err(CliParseErr::NoPids);
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-s", "KILL"];
        let expected = Err(CliParseErr::NoPids);
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill", "-s", "9"];
        let expected = Err(CliParseErr::NoPids);
        assert_eq!(parse_cli(args.into_iter()), expected);

        let args: Vec<String> = string_vec!["kill"];
        let expected = Err(CliParseErr::NoPids);
        assert_eq!(parse_cli(args.into_iter()), expected);
    }

    #[test]
    fn test_parse_cli_err_option_argument_expected() {
        let args: Vec<String> = string_vec!["kill", "-s"];
        let expected = Err(CliParseErr::ArgExpected("-s".to_owned()));
        assert_eq!(parse_cli(args.into_iter()), expected);
    }
}
