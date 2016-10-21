extern crate coreutils;


use coreutils::to_human_readable_string;
use std::env;
use std::fs::File;
use std::io::{stderr, Read, Write};
use std::time::Instant;

fn main() {
    let stderr = stderr();
    let mut stderr = stderr.lock();

    let mut bs = 512;
    let mut count = None;
    let mut in_path = String::new();
    let mut out_path = String::new();
    let mut status = 1;
    for arg in env::args().skip(1) {
        if arg.starts_with("bs=") {
            bs = arg[3..].parse::<usize>().expect("dd: bs is not a number");
        } else if arg.starts_with("count=") {
            count = Some(arg[6..].parse::<usize>().expect("dd: count is not a number"));
        } else if arg.starts_with("if=") {
            in_path = arg[3..].to_string();
        } else if arg.starts_with("of=") {
            out_path = arg[3..].to_string();
        } else if arg.starts_with("status=") {
            match &arg[7..] {
                "none" => status = 0,
                "noxfer" => status = 1,
                "progress" => status = 2,
                unknown => panic!("dd: status: unrecognized argument '{}'", unknown)
            }
        } else {
            panic!("dd: unrecognized operand '{}'", arg);
        }
    }

    let mut input = File::open(in_path).expect("dd: failed to open if");
    let mut output = File::create(out_path).expect("dd: failed to open of");

    let mut in_recs = 0;
    let mut in_extra = 0;
    let mut out_recs = 0;
    let mut out_extra = 0;
    let mut out_total = 0;
    let mut running = true;
    let mut buffer = vec![0; bs];
    let mut start = Instant::now();
    let mut last_print = 0;
    let mut last_print_out = 0;
    while running {
        let in_count = input.read(&mut buffer).expect("dd: failed to read if");
        if in_count < bs {
            if in_count > 0 {
                in_extra += 1;
            }
            running = false;
        } else {
            in_recs += 1;
            if let Some(count) = count {
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
