use std::env;

fn main() {
    println!("{}", env::current_dir().unwrap().display())
}
