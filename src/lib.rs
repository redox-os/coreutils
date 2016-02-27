#![deny(warnings)]
#![feature(core_intrinsics)]

pub mod extra {
    use std::process::exit;
    use std::error::Error;
    use std::io::{self, Write};

    /// Extension for Option-like types
    pub trait OptionalExt {
        /// The "success" variant of this optional type.
        type Succ;

        /// Unwrap or abort program with exit code
        fn try<W: Write>(self, stderr: &mut W) -> Self::Succ;

        /// Unwrap or abort the program with failed exit code and custom error message
        fn fail<'a, W: Write>(self, err: &'a str, stderr: &mut W) -> Self::Succ;

        /// An unwrapping where the fail-case is not checked and threaten as statical unreachable.
        unsafe fn unchecked_unwrap(self) -> Self::Succ;
    }

    impl<T, U: Error> OptionalExt for Result<T, U> {
        type Succ = T;

        fn try<W: Write>(self, stderr: &mut W) -> T {
            match self {
                Ok(succ) => succ,
                Err(e) => {
                    // We ignore the results to avoid stack overflow (because of unbounded
                    // recursion).
                    let _ = stderr.write(b"error: ");
                    let _ = stderr.write(e.description().as_bytes());
                    let _ = stderr.write(b"\n");
                    let _ = stderr.flush();
                    exit(1);
                },
            }
        }

        fn fail<'a, W: Write>(self, err: &'a str, stderr: &mut W) -> T {
            match self {
                Ok(succ) => succ,
                Err(_) => {
                    let _ = stderr.write(b"error: ");
                    let _ = stderr.write(err.as_bytes());
                    let _ = stderr.write(b"\n");
                    let _ = stderr.flush();
                    exit(1);
                },
            }
        }

        unsafe fn unchecked_unwrap(self) -> T {
            if let Ok(x) = self {
                x
            } else {
                unreachable()
            }
        }
    }

    impl<T> OptionalExt for Option<T> {
        type Succ = T;

        fn try<W: Write>(self, stderr: &mut W) -> T {
            match self {
                Some(succ) => succ,
                None => {
                    let _ = stderr.write(b"error: (no message)\n");
                    let _ = stderr.flush();
                    exit(1);
                },
            }
        }

        fn fail<'a, W: Write>(self, err: &'a str, stderr: &mut W) -> T {
            match self {
                Some(succ) => succ,
                None => {
                    let _ = stderr.write(b"error:");
                    let _ = stderr.write(err.as_bytes());
                    let _ = stderr.write(b"\n");
                    let _ = stderr.flush();
                    exit(1);
                },
            }
        }

        unsafe fn unchecked_unwrap(self) -> T {
            if let Some(x) = self {
                x
            } else {
                unreachable()
            }
        }
    }

    pub trait WriteExt {
        fn writeln(&mut self, s: &[u8]) -> io::Result<usize>;
    }

    impl<W: Write> WriteExt for W {
        fn writeln(&mut self, s: &[u8]) -> io::Result<usize> {
            let res = self.write(s).map(|x| x + 1);
            try!(self.write(b"\n"));

            res
        }
    }

    /// A hint which is threaten as statical unreachable in release mode, and panic (unreachable!())
    /// in debug mode.
    #[cfg(debug)]
    pub unsafe fn unreachable() -> ! {
        unreachable!();
    }


    /// A hint which is threaten as statical unreachable in release mode, and panic (unreachable!())
    /// in debug mode.
    #[cfg(not(debug))]
    pub unsafe fn unreachable() -> ! {
        use std::intrinsics::unreachable;

        unreachable();
    }

    #[macro_export]
    macro_rules! try_some {
        ($x:expr) => {
            if let Some(x) = $x {
                x
            } else {
                return None;
            }
        };
        ($x:expr => $y:expr) => {
            if let Some(x) = $x {
                x
            } else {
                return $y;
            }
        };
    }

    pub fn fail<'a, W: Write>(s: &'a str, stderr: &mut W) -> ! {
        let _ = stderr.write(b"error: ");
        let _ = stderr.write(s.as_bytes());
        let _ = stderr.write(b"\n");
        let _ = stderr.flush();
        exit(1);
    }
}
