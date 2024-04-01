extern crate arg_parser;
extern crate coreutils;
extern crate extra;
extern crate num;
extern crate failure;
#[macro_use] extern crate failure_derive;

#[cfg(test)] #[macro_use] extern crate proptest;
#[cfg(test)] use proptest::prelude::*;
use std::env;
use std::fs::File;
use std::io::{stderr, stdin, stdout, Read, Write};
use std::time::Instant;
use std::process::exit;
use std::str::FromStr;
use arg_parser::ArgParser;
use coreutils::to_human_readable_string;
use extra::option::OptionalExt;
use num::{PrimInt,Num,CheckedMul};
use failure::Error;

const MAN_PAGE: &'static str = /* @MANSTART{dd} */ r#"
NAME
    dd - copy a file

SYNOPSIS
    dd [ -h | --help ] if=[FILE] of=[FILE]

DESCRIPTION
    The dd tool copies from a file to another file in 512-byte block sizes

OPTIONS
    -h
    --help
        display this help and exit

    bs=n
        set input and output block size to n bytes
    count=n
        copy only n blocks
    if=file
        read from file instead of standard input
    of=file
        write output to file instead of standard out
    Any number can end with the suffix of b(512), k(1024), m(1024^2), or g(1024^3) bytes

"#; /* @MANEND */



fn main() {
    let stdin = stdin();
    let stdin = stdin.lock();
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut parser = ArgParser::new(6)
        .add_flag(&["h", "help"])
        .add_setting_default("bs", "512")
        .add_setting_default("count", "-1")
        .add_setting_default("status", "1")
        .add_setting("if")
        .add_setting("of");
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }
    //pretty panic
    macro_rules! pp {
        ($x:expr) => (
            get_int(parser.get_setting($x).unwrap()).unwrap_or_else(|message| {
                let _ = writeln!(stdout, "dd: {}",message);
                exit(1);
            })
        )
    }
    let bs: usize = pp!("bs");
    let count: i64 = pp!("count");
    let status: usize = pp!("status");

    let mut input: Box<dyn Read> = match parser.found("if") {
        true => {
            let path = parser.get_setting("if").unwrap();
            Box::new(File::open(path).unwrap_or_else(|message| {
                let _ = writeln!(stdout, "dd: Unable to open {}: {}",parser.get_setting("if").unwrap(),message);
                exit(1);
            }))

        },
        false => Box::new(stdin),
    };

    let mut output: Box<dyn Write> = match parser.found("of") {
        true => {
            let path = parser.get_setting("of").unwrap();
            Box::new(File::create(path).unwrap_or_else(|message| {
                let _ = writeln!(stdout, "dd: Unable to open {}: {}", parser.get_setting("of").unwrap(), message.to_string());
                exit(1);
            }))
        },
        false => Box::new(stdout),
    };

    let mut in_recs = 0;
    let mut in_extra = 0;
    let mut out_recs = 0;
    let mut out_extra = 0;
    let mut out_total = 0;
    let mut running = true;
    let mut buffer = vec![0; bs];
    let mut last_print = 0;
    let mut last_print_out = 0;
    let start = Instant::now();
    while running {
        let in_count = input.read(&mut buffer).expect("dd: failed to read if");
        if in_count < bs {
            if in_count > 0 {
                in_extra += 1;
            }
            running = false;
        } else {
            in_recs += 1;
            if count != -1 {
                if in_recs >= count {
                    running = false;
                }
            }
        }

        let out_count = output.write(&buffer[.. in_count]).expect("dd: failed to write of");
        if out_count < bs {
            if out_count > 0 {
                out_extra += 1;
            }
            running = false;
        } else {
            out_recs += 1;
        }

        out_total += out_count as u64;

        if status >= 2 {
            let elapsed = start.elapsed().as_secs();
            if elapsed > last_print {
                let _ = write!(stderr, "\r\x1B[K{} bytes ({}B) copied, {} s, {}B/s", out_total, to_human_readable_string(out_total), elapsed, to_human_readable_string(out_total - last_print_out));
                let _ = stderr.flush();
                last_print = elapsed;
                last_print_out = out_total;
            }
        }
    }

    if status >= 1 {
        let elapsed_duration = start.elapsed();
        let elapsed = elapsed_duration.as_secs() as f64 + (elapsed_duration.subsec_nanos() as f64)/1000000000.0;

        if status >= 2 && last_print > 0 {
            let _ = writeln!(stderr, "");
        }

        let _ = writeln!(stderr, "{}+{} records in", in_recs, in_extra);
        let _ = writeln!(stderr, "{}+{} records out", out_recs, out_extra);
        let _ = writeln!(stderr, "{} bytes ({}B) copied, {} s, {}B/s", out_total, to_human_readable_string(out_total), elapsed, to_human_readable_string((out_total as f64/elapsed) as u64));
    }
}

#[derive(Fail,Debug)]
enum DDerror {
    #[fail(display = "Parsing your number created an error: {}", _0)]
    ParseIntErr(String),
    #[fail(display = "Your number was too large and caused an overflow")]
    Overflow,
}


fn get_int<T>(mystr: String) -> Result<T, Error>
where T: PrimInt+Num+FromStr+CheckedMul,
      <T as std::str::FromStr>::Err : std::fmt::Debug+std::string::ToString,
{

    //f=from
    macro_rules! f {
        ( $x:expr )  => (
            T::from($x).unwrap()
            )
    }
    //Make a mutable copy
    let mut mutcopy: String = mystr;
    let ch = mutcopy.chars().rev().next().unwrap();
    let modifier = match ch {
        'b' => f!(512),
        'k' => f!(1024),
        'm' => f!(1024*1024),
        'g' => f!(1024*1024*1024),
        _ => T::one(),
    };

    if modifier == T::one() {
        let parsed = mutcopy.parse::<T>().map_err(|n| DDerror::ParseIntErr(n.to_string()))?;
        Ok(parsed)
    } else {
        mutcopy.pop();
        let parsed = mutcopy.parse::<T>().map_err(|n| DDerror::ParseIntErr(n.to_string()))?;
        let muld = modifier.checked_mul(&parsed).ok_or(DDerror::Overflow)?;
        Ok(muld)
    }


}

#[cfg(test)]
proptest! {
    //the s is a number, and the b is the byte, can be b,k,m,g or ""
    #[test]
    fn get_int_test(s in any::<i128>(), b in "[b,k,m,g]?") {
        let prefix: i128 = match b.as_str() {
            "b" => 512,
            "k" => 1024,
            "m" => 1024*1024,
            "g" => 1024*1024*1024,
            _ => 1,
        };
        let res: Result<i128, Error> = get_int::<i128>(s.to_string()+&b);
        if let Ok(r) = res {
            assert_eq!(r, s*prefix)
        }
    }
}
