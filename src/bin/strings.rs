// If this code works, it was written by Ticki. If it does not, I don't know who the hell wrote it
// but it was definitively not me. Blame someone else.

#[deny(warnings)]

use std::io::{self, Write, Read};
use std::error::Error;
use std::process::exit;

#[derive(Copy, Clone)]
struct Trailing {
    chars: [u8; 4],
    current: usize,
}

// Wow, such premature, much optimization
impl Trailing {
    #[inline]
    fn new() -> Trailing {
        Trailing {
            chars: [0; 4],
            current: 0,
        }
    }

    fn set(&mut self, b: u8) -> bool {
        self.chars[self.current] = b;
        self.current += 1;

        self.is_complete()
    }

    fn reset(&mut self) {
        self.current = 0;
    }

    fn is_complete(self) -> bool {
        self.current == 4
    }

    fn chars(self) -> [u8; 4] {
        self.chars
    }
}

trait IsPrintable {
    fn is_printable(self) -> bool;
}

impl IsPrintable for u8 {
    fn is_printable(self) -> bool {
        // TODO handle unicode.
        self >= 0x20 && self <= 0x7e
    }
}

trait Try {
    type Succ;

    fn try(self) -> Self::Succ;
}

impl<T, U: Error> Try for Result<T, U> {
    type Succ = T;

    fn try(self) -> T {
        match self {
            Ok(succ) => succ,
            Err(e) => {
                println!("error: {}", e.description());
                exit(128 + 1);
            },
        }
    }
}

fn main() {
    let mut trailing = Trailing::new();
    let mut stdout = io::stdout();

    for i in io::stdin().bytes().map(|x| x.try()) {
        if i.is_printable() {
            if trailing.is_complete() {
                stdout.write(&[i]).try();
            } else if trailing.set(i) {
                stdout.write(&trailing.chars()).try();
            }
        } else {
            if trailing.is_complete() {
                stdout.write(&[b'\n']).try();
            }
            trailing.reset();
        }
    }
}
