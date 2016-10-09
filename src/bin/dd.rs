use std::env;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let mut bs = 512;
    let mut count = None;
    let mut in_path = String::new();
    let mut out_path = String::new();
    for arg in env::args().skip(1) {
        if arg.starts_with("bs=") {
            bs = arg[3..].parse::<usize>().expect("dd: bs is not a number");
        } else if arg.starts_with("count=") {
            count = Some(arg[6..].parse::<usize>().expect("dd: count is not a number"));
        } else if arg.starts_with("if=") {
            in_path = arg[3..].to_string();
        } else if arg.starts_with("of=") {
            out_path = arg[3..].to_string();
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

        out_total += out_count;
    }

    println!("{}+{} records in", in_recs, in_extra);
    println!("{}+{} records out", out_recs, out_extra);
    println!("{} bytes copied", out_total);
}
