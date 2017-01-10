#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{self, Stdin, Stdout, Stderr, Write};
use std::cell::Cell;

use extra::io::fail;

static USAGE: &'static str = r#"usage: tr -c string1  string2
    tr -d string1
    tr -s string1 string2
"#;

const MAN_PAGE: &'static str = /* @MANSTART{tr} */ r#"NAME
    tr - translate characters

SYNOPSIS
    tr [ -cds ] [ string1 [ string2 ] ]

DESCRIPTION
    Tr copies the standard input to the standard output with
    substitution or deletion of selected characters.  Input
    characters found in string1 are mapped into the correspond-
    ing characters of string2. When string2 is short it is pad-
    ded to the length of string1 by duplicating its last charac-
    ter.  Any combination of the options -cds may be used:

    -c   complement the set of characters in string1 with
         respect to the universe of characters whose ASCII codes
         are 01 through 0377

    -d   delete all input characters in string1

    -s   squeeze all strings of repeated output characters that
         are in string2 to single characters

    -t   first truncate string1 to length of string2

    In either string the notation a-b means a range of charac-
    ters from a to b in increasing ASCII order.  The character
    `\' followed by 1, 2 or 3 octal digits stands for the char-
    acter whose ASCII code is given by those digits.  A `\' fol-
    lowed by any other character stands for that character.

    The following example creates a list of all the words in
    `file1' one per line in `file2', where a word is taken to be
    a maximal string of alphabetics.  The second string is
    quoted to protect `\' from the Shell.  012 is the ASCII code
    for newline.

        tr -cs A-Za-z '\012' <file1 >file2

SEE ALSO
    ed(1), ascii(7)

BUGS
    loads.
"#; /* @MANEND */


struct Translation {
    complement:  bool,
    delete:      bool,
    squeeze:     bool,
    truncate:    bool,
    search:      String,
    replace:     String,
    status:      Cell<i32>
}

impl Translation {
    fn print_opts(&self) {
        println!("flags\ncompliment {}\ndelete: {}\nsqueeze {}", self.complement, self.delete, self.squeeze);
        println!("search: {}", self.search);
        println!("replace: {}", self.replace);
    }

    fn append_or_truncate(&mut self) -> &mut Translation {
        // first decide
        let search_length = self.search.chars().count();
        let replace_length = self.replace.chars().count();

        if replace_length < search_length {
            // adjust search or replace?
            if self.truncate {
                // truncate search to replace's length
                // use an iterator just in case we have diacretes or other complex chars
                let mut new_search = "".to_string();
                {
                    let mut char_walker = self.search.chars();
                    for _ in 0 .. replace_length {
                        let last_char = char_walker.next().unwrap();
                        new_search.push(last_char);
                    }
                }
                self.search = new_search;
            } else {
                // fill replace with it's last char to match search in length
                let lastchar_as_string = self.replace.chars().last().unwrap();
                for _ in replace_length .. search_length {
                    self.replace.push(lastchar_as_string); // do something
                }
            }
        }
        return self;
    }

    fn get_opts(&mut self, stdout: &mut Stdout, mut stderr: &mut Stderr) -> &mut Translation {
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                match arg.as_str() {
                    "-c" => {
                        self.complement = true;
                    }
                    "-d" => {
                        self.delete = true;
                    }
                    "-s" => {
                        self.squeeze = true;
                    }
                    "-t" => {
                        self.truncate = true;
                    }
                    "-h" | "--help" => {
                        let _ = stdout.write(MAN_PAGE.as_bytes());
                    }
                    _ => fail("invalid option", &mut stderr),
                }
            } else {
                if self.search.is_empty() {
                    self.search = arg;
                } else {
                    self.replace = arg;
                }
            }
        }
        return self;
    }
    fn check_opts(&mut self) -> &mut Translation {
        if !self.delete && !self.squeeze && self.replace.len() == 0 {
            // big issue
            println!("replace string can not be empty when neither -s nor -d is specified.");
            println!("{}", MAN_PAGE);
            self.status.set(1);
        }

        return self;
    }
    fn translate(&self, _stdin: Stdin, mut _stdout: &mut Stdout, mut _stderr: &mut Stderr) {
        // do the work
    }
}

#[cfg(not(test))]
fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mut tr = Translation { complement: false, delete: false, squeeze: false, truncate: false, search: String::new(), replace: String::new(), status: Cell::new(0)};
    tr.get_opts(&mut stdout,&mut stderr);
    tr.check_opts();
// actually put them somewhere for retrieval by the other parts of the program instead of print
    tr.append_or_truncate();
    tr.print_opts();
    if tr.status.get() > 0 {
        fail(USAGE, &mut stderr);
    }

    // if complement is turned on recreate 'search' to contain the complement of search
    tr.translate(stdin, &mut stdout, &mut stderr);
    // open std input
    // open std ouput
// read a char
// if not in search => pass through
// decide what to do
// switching over:
// case either 'delete' switched on or find matching char in 'replace'
// case 'squeeze' switched on
// 
}

#[cfg(test)]
mod tests {
    use super::Translation;

    #[test]
    fn append_replace_when_it_is_short() {
        let mut tr = Translation {search: "abcde".to_string() , replace: "xyz".to_string(), complement: false, delete: false, squeeze: false};
        tr.append_replace();
        assert_eq!("xyzzz", tr.replace);
    }
    #[test]
    fn append_replace_when_it_is_long() {
        let mut tr = Translation {search: "a".to_string() , replace: "xyz".to_string(), complement: false, delete: false, squeeze: false};
        tr.append_replace();
        assert_eq!("x", tr.replace);
    }
    #[test]
    fn append_replace_when_it_is_exact_in_length() {
        let mut tr = Translation {search: "abc".to_string() , replace: "xyz".to_string(), complement: false, delete: false, squeeze: false};
        tr.append_replace();
        assert_eq!("xyz", tr.replace);
    }
}
