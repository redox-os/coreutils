#![deny(warnings)]
#![feature(core_intrinsics)]

pub mod extra {
    use std::process::exit;
    use std::error::Error;
    use std::io::{stdout, Write};

    /// Extension for Option-like types
    pub trait OptionalExt {
        /// The "success" variant of this optional type.
        type Succ;

        /// Unwrap or abort program with exit code
        fn try(self) -> Self::Succ;

        /// Unwrap or abort the program with failed exit code and custom error message
        fn fail<'a>(self, err: &'a str) -> Self::Succ;

        /// An unwrapping where the fail-case is not checked and threaten as statical unreachable.
        unsafe fn unchecked_unwrap(self) -> Self::Succ;
    }

    impl<T, U: Error> OptionalExt for Result<T, U> {
        type Succ = T;

        fn try(self) -> T {
            match self {
                Ok(succ) => succ,
                Err(e) => {
                    let mut stdout = stdout();
                    // We ignore the results to avoid stack overflow (because of unbounded
                    // recursion).
                    let _ = stdout.write(b"error: ");
                    let _ = stdout.write(e.description().as_bytes());
                    let _ = stdout.write(b"\n");
                    let _ = stdout.flush();
                    exit(1);
                },
            }
        }

        fn fail<'a>(self, err: &'a str) -> T {
            match self {
                Ok(succ) => succ,
                Err(_) => {
                    let mut stdout = stdout();
                    let _ = stdout.write(b"error: ");
                    let _ = stdout.write(err.as_bytes());
                    let _ = stdout.write(b"\n");
                    let _ = stdout.flush();
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

        fn try(self) -> T {
            match self {
                Some(succ) => succ,
                None => {
                    // Why not use println!?
                    // The reason is that we want optimal performance, which we cannot get, when
                    // using println, due to the fact that it goes through another layer of
                    // indirection formatters: Formatters will do an unneccary allocation, which we
                    // do not need in this case.
                    let mut stdout = stdout();
                    let _ = stdout.write(b"error: (no message)\n");
                    let _ = stdout.flush();
                    exit(1);
                },
            }
        }

        fn fail<'a>(self, err: &'a str) -> T {
            match self {
                Some(succ) => succ,
                None => {
                    let mut stdout = stdout();
                    let _ = stdout.write(b"error:");
                    let _ = stdout.write(err.as_bytes());
                    let _ = stdout.write(b"\n");
                    let _ = stdout.flush();
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

    pub fn fail<'a>(s: &'a str) -> ! {
        use std::io::{Write, stdout};

        let mut stdout = stdout();
        let _ = stdout.write(b"error: ");
        let _ = stdout.write(s.as_bytes());
        let _ = stdout.write(b"\n").try();
        let _ = stdout.flush();
        exit(1);
    }
}

