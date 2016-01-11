use std::env;

fn main() {
    let answer = env::args().skip(1).next().unwrap_or("y".into());

    loop {
        println!("{}", answer)
    }
}
