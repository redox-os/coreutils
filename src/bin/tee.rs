//#![deny(warnings)]

extern crate coreutils;

use coreutils::ArgParser;
use std::{process, env};
use std::io::{self, Read, Write};

const MAN_PAGE: &'static str = /* @MANSTART{tee} */ r#"
NAME
    tee - read from standard input and write to standard output and files

SYNOPSIS
    tee [OPTION]... [FILE]...

DESCRIPTION
    Copy standard input to each FILE, and also to standard output.

    -a, --append
        append to given FILEs, do not overwrite

    --help display this help and exit

AUTHOR
    Written by Stefan Lücke.
"#; /* @MANEND */

fn main() {
    let mut parser = ArgParser::new(2).
        add_flag(&["a", "append"]).
        add_flag(&["h", "help"]);
    parser.parse(env::args());

    let mut stdout = io::stdout();

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).unwrap();
        stdout.flush().unwrap();
        process::exit(0);
    }

    let mut fds: Vec<std::fs::File> = Vec::with_capacity(env::args().len());

    if parser.found("append") {
        let args = env::args().skip(2);
        if args.len() > 0 {
            for arg in args {
               let fd = std::fs::OpenOptions::new().append(true).open(arg);
               match fd {
                   Ok(f) => fds.push(f),
                   Err(e) => println!("Err(e): {}", e),
               }
            }
        }
    } else {
        let args = env::args().skip(1);
        if args.len() > 0 {
            for arg in args {
               let fd = std::fs::OpenOptions::new().write(true).create(true).open(arg);
               match fd {
                   Ok(f) => fds.push(f),
                   Err(e) => println!("Err(e): {}", e),
               }
            }
        }
    }

    let stdintemp = io::stdin();
    let mut stdin = stdintemp.lock();
    let mut buffer: &mut [u8] = &mut[0 as u8; 4096];
    'programmloop: loop {
        let result_read = stdin.read(buffer);
        match result_read {
            Ok(size) => {
                if size == 0 {
                    // we've read a EOF here
                    break;
                }

                let result_write = stdout.write(&mut buffer[0..size]);
                    match result_write {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Err(e): {}", e);
                            break 'programmloop;
                        },
                    };

                // iterate over open files
                'writeloop: for mut f in &mut fds {
                    let result_write = f.write(&mut buffer[0..size]);
                    match result_write {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Err(e): {}", e);
                            break 'programmloop;
                        },
                    };
                }

            },
            Err(e) => {
                println!("Err(e): {}", e);
                break;
            },
        };
    }

    stdout.flush().unwrap();
    process::exit(0);
}
