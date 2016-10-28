#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Param {
    Short(char),
    Long(String),
}

pub trait IntoParam {
    fn into_param(self) -> Param;
}

impl IntoParam for &'static str {
    fn into_param(self) -> Param {
        Param::Long(self.to_owned())
    }
}

impl IntoParam for char {
    fn into_param(self) -> Param {
        Param::Short(self)
    }
}

use std::collections::HashMap;

/// Our homebrewed Arg Parser
///
/// Yes, it would be nice to use an Arg Parser library but we don't have that
/// kind of luxury with our current std crate implementation for Redox
#[derive(Clone, Debug, Default)]
pub struct ArgParser {
    flags: HashMap<Param, bool>,
    opts:  HashMap<Param, Option<String>>,
    pub args: Vec<String>,
}

impl ArgParser {
    /// Create a new ArgParser object
    pub fn new(flag_cap: usize, opt_cap: usize) -> Self {
        ArgParser {
            flags: HashMap::with_capacity(flag_cap),
            opts: HashMap::with_capacity(opt_cap),
            args: Vec::new(),
        }
    }

    /// Builder method for adding both short and long flags
    pub fn add_flag(mut self, short: &str, long: &str) -> Self {
        if let Some(short) = short.chars().next() {
            self.flags.insert(Param::Short(short), false);
        }
        if !long.is_empty() {
            self.flags.insert(Param::Long(long.to_owned()), false);
        }
        self
    }

    /// Builder method for adding both short and long opts
    pub fn add_opt(mut self, short: &str, long: &str) -> Self {
        if let Some(short) = short.chars().next() {
            self.opts.insert(Param::Short(short), None);
        }
        if !long.is_empty() {
            self.opts.insert(Param::Long(long.to_owned()), None);
        }
        self
    }

    /// Check if a flag has been used
    pub fn enabled_flag<F: IntoParam>(&self, flag: F) -> bool {
        *self.flags.get(&flag.into_param()).unwrap_or(&false)
    }

    /// Start parsing user inputted args for which flags are used
    pub fn initialize<A: Iterator<Item=String>>(&mut self, args: A) {
        let mut args = args.skip(1);
        while let Some(mut arg) = args.next() {
            if arg.starts_with("--") {
                // Remove both dashes
                arg.remove(0);
                arg.remove(0);
                if let Some(i) = arg.find('=') {
                    let (lhs, rhs) = arg.split_at(i);
                    if let Some(opt) = self.opts.get_mut(&Param::Long(lhs.to_owned())) {
                        *opt = Some(rhs.to_owned());
                    }
                }
                else {
                    if let Some(flag) = self.flags.get_mut(&Param::Long(arg)) {
                        *flag = true;
                    }
                }
            }
            else if arg.starts_with("-") {
                for ch in arg[1..].chars() {
                    if let Some(switch) = self.flags.get_mut(&Param::Short(ch)) {
                        *switch = true;
                    }
                    if let Some(value) = self.opts.get_mut(&Param::Short(ch)) {
                        *value = args.next();
                    }
                }
            }
            else {
                self.args.push(arg);
            }
        }
    }

    pub fn set_flag<F: IntoParam>(&mut self, flag: F, state: bool) {
        if let Some(switch) = self.flags.get_mut(&flag.into_param()) {
            *switch = state;
        }
    }

    pub fn set_opt<O: IntoParam>(&mut self, opt: O, state: Option<String>) {
        if let Some(value) = self.opts.get_mut(&opt.into_param()) {
            *value = state;
        }
    }

    pub fn has_opt<O: IntoParam>(&mut self, opt: O) -> bool {
        if let Some(&Some(_)) = self.opts.get(&opt.into_param()) {
            return true;
        }
        false
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
