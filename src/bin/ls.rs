#[macro_use]
extern crate clap;

use clap::{App, Arg};

fn main() {
    let opts = App::new("ls")
                   .version("0.1")
                   .author("Redox OS developers")
                   .about("List files")
                   .arg(Arg::with_name("long")
                            .short("l")
                            .long("long")
                            .help("Use long display format"))
                   .arg(Arg::with_name("onecol")
                            .short("1")
                            .help("Display files in one column"))
                   .arg(Arg::with_name("all")
                            .short("a")
                            .long("all")
                            .help("Show hidden files"))
                   .get_matches();

    // TODO
}
