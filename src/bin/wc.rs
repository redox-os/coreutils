use std::env;
use std::io;
use std::io::Read;
use std::fs::File;

fn main() {
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_bytes = false;
    let mut arg_idx = 1;
    let mut first_file_idx = 0;

    for arg in env::args().skip(1) {
        //TODO match things like -cl
        //maybe also long args? 
        //ideally we would have getopt[s] equiv
        //TODO add -m, maybe add -L
        match &*arg {
            "-l" => count_lines = true,
            "-w" => count_words = true,
            "-c" => count_bytes = true,
            _ => {
                first_file_idx = arg_idx;
                break;
            }
        };

        arg_idx += 1;
    }

    //defaults to behavior of -lwc
    if !(count_lines || count_words || count_bytes) {
        count_lines = true;
        count_words = true;
        count_bytes = true;
    }

    if first_file_idx == 0 {
        //FIXME perhaps I'm using this wrong
        //or perhaps needs to be fixed in ion
        //but this terminates on \n rather than EOF
        let (lines, words, bytes) = do_count(&mut io::stdin());

        print!("\t");

        if count_lines {
            print!("{} ", lines);
        }
        if count_words {
            print!("{} ", words);
        }
        if count_bytes {
            print!("{} ", bytes);
        }

        println!("");
    } else {
        let mut total_lines = 0;
        let mut total_words = 0;
        let mut total_bytes = 0;

        for path in env::args().skip(arg_idx) {
            //TODO would be easy to use stdin for - but
            //that is probably something the shell should handle?
            //unix it's all just fds so it's whatever dunno here tho
            //(also - is specific to sh/bash fwiw)
            match File::open(&path) {
                Ok(mut file) => {
                    let (lines, words, bytes) = do_count(&mut file);

                    total_lines += lines;
                    total_words += words;
                    total_bytes += bytes;

                    print!("\t");

                    if count_lines {
                        print!("{} ", lines);
                    }
                    if count_words {
                        print!("{} ", words);
                    }
                    if count_bytes {
                        print!("{} ", bytes);
                    }

                    println!("{}", path);
                },
                Err(err) => println!("wc: cannot open file {}: {}", path, err)
            }
        }

        if env::args().len() - arg_idx > 1 {
            //XXX this is copy-pasted in two other places
            //make it a fn or something
            print!("\t");

            if count_lines {
                print!("{} ", total_lines);
            }
            if count_words {
                print!("{} ", total_words);
            }
            if count_bytes {
                print!("{} ", total_bytes);
            }

            println!("Total");
        }
    }
}

fn is_whitespace(byte: &u8) -> bool {
    //FIXME this works like iswspace w/ default C locale
    //but not good enough for en_US.UTF8 among others
    *byte == b'\n'
    || *byte == b'\t'
    || *byte == b'\r'
    || *byte == 0xc //formfeed
    || *byte == 0xb //vtab
    || *byte == b' '
}

fn do_count<T: std::io::Read>(input: &mut T) -> (i32, i32, i32) {
    let mut line_count = 0;
    let mut word_count = 0;
    let mut byte_count = 0;
    let mut got_space = true;

    for byte_result in input.bytes() {
        if let Ok(byte) = byte_result {
            if byte == b'\n' {
                line_count += 1;
            }

            if is_whitespace(&byte) {
                got_space = true;
            } else if got_space {
                got_space = false;
                word_count += 1;
            }

            byte_count += 1;
        }
    }

    (line_count, word_count, byte_count)
}
