//#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::Command;
use std::time::Instant;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{time} */ r#"
NAME
    time - timer for commands
    
SYNOPSIS
    time [COMMAND] [ARGUEMENT]...
    
DESCRIPTION
    Runs the command taken as the first arguement and outputs the time the command took to execute.
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let time = Instant::now();

    let mut args = env::args().skip(1);
    if let Some(name) = args.next() {
        let mut command = Command::new(&name);
        for arg in args {
            command.arg(&arg);
        }
        command.spawn().try(&mut stderr).wait().try(&mut stderr);
    }

    let duration = time.elapsed();
    stdout.write(&format!("{}m{:.3}s\n", duration.as_secs() / 60,
                                   (duration.as_secs()%60) as f64 + (duration.subsec_nanos() as f64)/1000000000.0
                ).as_bytes()).try(&mut stderr);
}
