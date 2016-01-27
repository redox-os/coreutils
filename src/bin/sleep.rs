use std::env;
use std::time::Duration;
use std::thread;

//TODO: Make redox use u64, u32
#[cfg(target_os="redox")]
fn sleep_hack(seconds: u64) {
    thread::sleep(Duration::new(seconds as i64, 0))
}

#[cfg(not(target_os="redox"))]
fn sleep_hack(seconds: u64) {
    thread::sleep(Duration::new(seconds, 0))
}

fn main() {
    if let Some(arg) = env::args().nth(1) {
        match arg.parse::<u64>() {
            Ok(seconds) => sleep_hack(seconds),
            Err(err) => println!("sleep: invalid argument: {}", err),
        }
    } else {
        println!("sleep: missing argument");
    }
}
