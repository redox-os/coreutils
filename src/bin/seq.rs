use std::env;
use std::process;

fn main() {
    if env::args().count() < 2 {
        println!("seq: Missing argument!");
        println!("Example: seq [VALUE]");
        process::exit(1);
    }

    let max: u32 = match std::env::args().nth(1).map(|a| a.parse()) {
        Some(Ok(n)) if n > 0 => n,
        _ => {
            println!("Invalid value: please provide a valid, unsigned number.");
            process::exit(1);
        }
    };

    for i in 1..max + 1 {
        println!("{}", i);
    }
}
