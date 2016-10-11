#[derive(Clone, Debug)]
pub enum Flag {
    Short(char),
    Long(&'static str),
}

/// Our homebrewed Arg Parser
///
/// Yes, it would be nice to use an Arg Parser library but we don't have that
/// kind of luxury with our current std crate implementation for Redox
#[derive(Clone, Debug, Default)]
pub struct ArgParser {
    flags: Vec<(Option<char>, Option<String>, bool)>,
    pub args: Vec<String>,
}

impl ArgParser {
    /// Create a new ArgParser object
    pub fn new(flag_amount: usize) -> Self {
        ArgParser { flags: Vec::with_capacity(flag_amount), args: Vec::new(), }
    }

    /// Builder method for adding both short and long flags
    pub fn add_flag(mut self, short: &str, long: &str) -> Self {
        if short.is_empty() && long.is_empty() {
            return self;
        }
        let short = if !short.is_empty() { short.chars().next() } else { None };
        let long = if !long.is_empty() { Some(long.to_owned()) } else { None };
        self.flags.push((short, long, false));
        self
    }

    /// Check if a flag has been used
    pub fn enabled_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Short(short) => {
                for &(ref parsed, _, ref switch) in self.flags.iter() {
                    if Some(short) == *parsed {
                        return *switch;
                    }
                }
            }
            Flag::Long(long) => {
                for &(_, ref parsed, ref switch) in self.flags.iter() {
                    if Some(long.to_owned()) == *parsed {
                        return *switch;
                    }
                }
            }
        }
        false
    }

    /// Start parsing user inputted args for which flags are used
    pub fn initialize<A: IntoIterator<Item=String>>(&mut self, args: A) {
        for mut arg in args.into_iter().skip(1) {
            if arg.starts_with("--") {
                // Remove both dashes
                arg.remove(0);
                arg.remove(0);
                for &mut (_, ref long, ref mut switch) in self.flags.iter_mut() {
                    if Some(arg.clone()) == *long {
                        *switch = true;
                    }
                }
            }
            else if arg.starts_with("-") {
                for ch in arg[1..].chars() {
                    for &mut (ref short, _, ref mut switch) in self.flags.iter_mut() {
                        if Some(ch) == *short {
                            *switch = true;
                        }
                    }
                }
            }
            else {
                self.args.push(arg);
            }
        }
    }
}

pub fn to_human_readable_string(size: u64) -> String {
    if size < 1024 {
        return format!("{}", size);
    }

    static UNITS: [&'static str; 7] = ["", "K", "M", "G", "T", "P", "E"];

    let sizef = size as f64;
    let digit_groups = (sizef.log10() / 1024f64.log10()) as i32;
    format!("{:.1}{}",
            sizef / 1024f64.powf(digit_groups as f64),
            UNITS[digit_groups as usize])
}
