#![deny(warnings)]
extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::fs::FileType;
use std::path::Path;
use std::io::{stdout, stderr, Stderr, Write, BufWriter};
use std::os::unix::fs::MetadataExt;
use std::process::exit;
use std::vec::Vec;

use coreutils::{ArgParser, to_human_readable_string, format_system_time};
use coreutils::columns::print_columns;
use extra::option::OptionalExt;


const MAN_PAGE: &'static str = /* @MANSTART{ls} */ r#"
NAME
    ls - list directory contents

SYNOPSIS
    ls [ -h | --help | -l ] [FILE]...

DESCRIPTION
    List information about the FILE(s), or the current directory

OPTIONS
    -a, --all
        do not ignore entries starting with .
    -h, --human-readable
        with -l, print human readable sizes
    --help
        display this help and exit
    -l
        use a long listing format
    -r, --reverse
        reverse order while sorting
    -R, --recursive
        list subdirectories recursively
    --mdate --modified-date
        display date of last modification
    --adate --accessed-date
        display date of last access
    --cdate --created-date
        display date of creation

"#; /* @MANEND */

fn mode_to_human_readable(file_type: &FileType, symlink_file_type: &FileType, mode: u32) -> String {

    let mut result = String::from("");
    if symlink_file_type.is_symlink() {
        result.push('l')
    } else if file_type.is_dir() {
        result.push('d');
    } else {
        result.push('-');
    }

    let mode_str = format!("{:>6o}", mode);
    let mode_chars = mode_str[3..].chars();
    for i in mode_chars {
        match i {
            '7' => result.push_str("rwx"),
            '6' => result.push_str("rw-"),
            '5' => result.push_str("r-x"),
            '4' => result.push_str("r--"),
            '3' => result.push_str("-wx"),
            '2' => result.push_str("-w-"),
            '1' => result.push_str("--x"),
            _ => result.push_str("---"),
        }
    }

    return result;
}

fn print_item(item_path: &str, parser: &ArgParser, output: &mut Vec<String>, stderr: &mut Stderr) {
    
    let mut link_error = "";
    let symlink_metadata = fs::symlink_metadata(&item_path).try(stderr);
    let metadata = match fs::metadata(&item_path) {
        Ok(metadata) => metadata,
        Err(_) => {
            link_error = "broken link";
            fs::symlink_metadata(&item_path).try(stderr)
        }
    };
    if parser.found("long-format") {
        output.push(format!("{} {:>5} {:>5} ",
                              mode_to_human_readable(&(metadata.file_type()), &(symlink_metadata.file_type()), metadata.mode()),
                              metadata.uid(),
                              metadata.gid())
                    );
        if parser.found("human-readable") {
            output.push(format!("{:>6} ", to_human_readable_string(metadata.size())));
        } else {
            output.push(format!("{:>8} ", metadata.size()));
        }
    }
    if parser.found("modified-date") || parser.found("long-format") {
        let mtime = match metadata.modified(){
            Ok(mtime) => format_system_time(mtime),
            Err(_) => "mdate err".to_string(),
        };
        output.push(format!("{:>20} ", mtime));
    }
    if parser.found("accessed-date") {
        let atime = match metadata.accessed(){
            Ok(atime) => format_system_time(atime),
            Err(_) => "adate err".to_string(),
        };
        output.push(format!("{:>20} ", atime));
    }
    if parser.found("created-date") {
        let ctime = match metadata.created(){
            Ok(ctime) => format_system_time(ctime),
            Err(_) => "cdate err".to_string(),
        };
        output.push(format!("{:>20} ", ctime));
    }


    if item_path.starts_with("./") {
        output.push(item_path[2..].to_string());
    } else {
        output.push(item_path.to_string());
    }
    if parser.found("long-format") && symlink_metadata.file_type().is_symlink() {
        let symlink_target = fs::read_link(item_path)
            .expect("can't read link")
            .into_os_string()
            .into_string()
            .expect("can't get path as string");
        output.push(format!(" -> {}", symlink_target));
        if !link_error.is_empty() {
            output.push(format!(" ({})", link_error));
        }
    }
}

fn list_dir(path: &str, parser: &ArgParser, output: &mut Vec<String>, stderr: &mut Stderr) {
    let show_hidden = parser.found("all");

    let metadata = fs::metadata(path).try(stderr);
    if metadata.is_dir() {
        let read_dir = Path::new(path).read_dir().try(stderr);

        let mut entries: Vec<String> = read_dir.filter_map(|x| x.ok())
            .map(|x| {
                let file_name = x.file_name().to_string_lossy().into_owned();
                file_name
            })
            .filter(|x| show_hidden || !x.starts_with("."))
            .collect();

        if parser.found("reverse") {
            entries.sort_by(|a, b| b.cmp(a));
        } else {
            entries.sort_by(|a, b| a.cmp(b));
        }

        for entry in entries.iter() {
            let mut entry_path = path.to_owned();
            if !entry_path.ends_with('/') {
                entry_path.push('/');
            }
            entry_path.push_str(&entry);
            print_item(&entry_path, &parser, output, stderr);
            if parser.found("recursive") && metadata.is_dir() {
                list_dir(&entry_path, parser, output, stderr);
            }
        }
    } else {
        print_item(&path, &parser, output, stderr);
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    let mut stderr = stderr();

    let mut parser = ArgParser::new(6)
        .add_flag(&["a", "all"])
        .add_flag(&["l", "long-format"])
        .add_flag(&["h", "human-readable"])
        .add_flag(&["r", "reverse"])
        .add_flag(&["R", "recursive"])
        .add_flag(&["mdate", "modified-date"])
        .add_flag(&["adate", "accessed-date"])
        .add_flag(&["cdate", "created-date"])
        .add_flag(&["", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }


    let mut output = Vec::new();
    if parser.args.is_empty() {
        list_dir(".", &parser, &mut output, &mut stderr);
    } else {
        for dir in parser.args.iter() {
            list_dir(&dir, &parser, &mut output, &mut stderr);
        }
    }

    if parser.found("long-format") {
        for (i, word) in output.iter().enumerate() {
            stdout.write(word.as_bytes()).try(&mut stderr);
            if i % 4 == 3 {
                !stdout.write("\n".as_bytes()).try(&mut stderr);
            }
        }
    } else {
        print_columns(output);
    }
    stdout.flush().try(&mut stderr);
}

/*
fn print_columns(words: Vec<String>) {
    let stdout = stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    let mut stderr = stderr();

    let terminal_size = termion::terminal_size().unwrap().0 as usize;
    let columned = make_columns(words, terminal_size);
    for i in 0..columned[0].len() {
        for j in 0..columned.len() {
            if i < columned[j].len() {
                stdout.write(columned[j][i].as_bytes()).try(&mut stderr);
            }
        }
        stdout.write("\n".as_bytes()).try(&mut stderr);
    }
    stdout.flush().try(&mut stderr);
}


fn make_columns(mut words: Vec<String>, terminal_width: usize) -> Vec<Vec<String>> {

    let word_lengths: Vec<usize> = 
        words.iter().map(|x: &String| -> usize {(&x).len() + 2}).collect();

    let columns_amt = bin_search( word_lengths.iter().fold(0   , |x, y| {min(x, *y)})
                                , word_lengths.iter().fold(1000, |x, y| {max(x, *y)})
                                , &word_lengths
                                , terminal_width);

    let longest_words: Vec<usize> = 
        split_into_columns( &word_lengths
                          , columns_amt
                          , (words.len() / columns_amt) + 1
                          ).iter().map(longest_word).collect();

    let mut words_with_space: Vec<String> = Vec::new();
    let lines_amt = (words.len() / columns_amt) + 1;
    let mut longest_words_rep = Vec::new();
    for longest_word in longest_words {
        for _ in 0..lines_amt {
            longest_words_rep.push(longest_word.clone());
        }
    }

    for i in 0..words.len() {
        
        let whitespace = " ".repeat(longest_words_rep[i] - words[i].len());
        words[i].push_str(whitespace.as_str());
        words_with_space.push(words[i].clone());
    }

    split_into_columns::<String>(&words_with_space, columns_amt, (words.len() / columns_amt) + 1)
}



fn bin_search(min: usize, max: usize, words: &Vec<usize>, terminal_width: usize) -> usize {
    let diff = min as isize - max as isize;
    let fits = try_rows(words, min + (max - min) / 2, terminal_width);
    if  diff == -1 || diff == 0 || diff == 1{
        return min;
    } else if fits {
        return bin_search(min + (max - min) / 2, max, words, terminal_width);
    } else {
        return bin_search(min, min + (max - min) / 2, words, terminal_width);
    }
}

fn longest_word(words: &Vec<usize>) -> usize {
    words.iter().fold(0, |x, y| {max(x, *y)})
}

fn split_into_columns<T: Clone>(words: &Vec<T>, columns: usize, lines_amt: usize) -> Vec<Vec<T>> {
    let mut outputs: Vec<Vec<T>> = Vec::new();
    for _ in 0..columns {
        outputs.push(Vec::new())
    }
    let mut i = 0;
    'outer: for output in &mut outputs {
        let mut j = 0;
        while j < lines_amt {
            if i >= words.len() {
                break 'outer;
            }
            output.push(words.get(i).unwrap().clone());
            i += 1;
            j += 1;
        }
    }
    outputs
}

fn try_rows(input: &Vec<usize>, columns: usize, width: usize) -> bool {

    let lines_amt = (input.len() / columns) + 1;

    let sum_line_widths = |widths_list: Vec<usize>| -> usize {
        widths_list.iter().fold(0, |x, y| { x + *y})
    };

    sum_line_widths(
        split_into_columns(input, columns, lines_amt).iter().map(
            longest_word
        ).collect()
    ) <= width
}
*/

#[test]
fn test_human_readable() {
    assert_eq!(to_human_readable_string(0), "0");
    assert_eq!(to_human_readable_string(1023), "1023");
    assert_eq!(to_human_readable_string(1024), "1.0K");
    assert_eq!(to_human_readable_string(1024 + 100), "1.1K");
    assert_eq!(to_human_readable_string(1024u64.pow(2) * 2), "2.0M");
    assert_eq!(to_human_readable_string(1024u64.pow(3) * 3), "3.0G");
    assert_eq!(to_human_readable_string(1024u64.pow(4) * 4), "4.0T");
    assert_eq!(to_human_readable_string(1024u64.pow(5) * 5), "5.0P");
    assert_eq!(to_human_readable_string(1024u64.pow(6) * 6), "6.0E");
}

#[test]
fn test_format_system_time() {
    use std::ops::Add;
    use std::time::{SystemTime, Duration};
    let now = SystemTime::now();
    let future = SystemTime::now().add(Duration::from_secs(10));
    assert_ne!(format_system_time(now), format_system_time(future));
    // compare up to ten minutes: 2017-03-21 17:1_:__
    assert_eq!(format_system_time(now)[..15], format_system_time(future)[..15]);
}
