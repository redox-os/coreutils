#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Param {
    Short(char),
    Long(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Value {
    Flag(bool),
    Opt(Option<String>),
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
#[derive(Clone, Debug, Default)]
pub struct ArgParser {
    params: HashMap<Param, Value>,
    pub args: Vec<String>,
}

impl ArgParser {
    /// Create a new ArgParser object
    ///
    /// The capacity specifies the initial capacity of the number parameters.
    /// Always good to set it at the number of flags and opts total.
    pub fn new(capacity: usize) -> Self {
        ArgParser {
            params: HashMap::with_capacity(capacity),
            args: Vec::new(),
        }
    }

    /// Builder method for adding both short and long flags
    ///
    /// Flags are just parameters that have no assigned values. They are used
    /// for when certain features or options have been enabled for the application
    ///
    /// For example
    /// > ls -l --human-readable
    ///   ^  ^  ^
    ///   |  |  |
    ///   |  |  `-- A long flag to enable human readable numbers.
    ///   |  `-- A short flag to enable the long format.
    ///   `-- The command to list files.
    pub fn add_flag(mut self, short: &str, long: &str) -> Self {
        if let Some(short) = short.chars().next() {
            self.params.insert(Param::Short(short), Value::Flag(false));
        }
        if !long.is_empty() {
            self.params.insert(Param::Long(long.to_owned()), Value::Flag(false));
        }
        self
    }

    /// Builder method for adding both short and long opts
    ///
    /// Opts are parameters that hold assigned values. They are used
    /// for when certain features or options have been enabled for the application
    ///
    /// For example
    /// > ls -T 4 --color=always
    ///   ^  ^    ^
    ///   |  |    |
    ///   |  |    `-- A long opt to enable the use of color with value `always`.
    ///   |  `-- A short opt to set tab size to the value `4`.
    ///   `-- The command to list files.
    pub fn add_opt(mut self, short: &str, long: &str) -> Self {
        if let Some(short) = short.chars().next() {
            self.params.insert(Param::Short(short), Value::Opt(None));
        }
        if !long.is_empty() {
            self.params.insert(Param::Long(long.to_owned()), Value::Opt(None));
        }
        self
    }

    /// Start parsing user inputted args for which flags and opts are used at
    /// runtime. The rest of the args that are not associated to opts get added
    /// to `ArgParser.args`.
    pub fn initialize<A: Iterator<Item=String>>(&mut self, args: A) {
        let mut args = args.skip(1);
        while let Some(mut arg) = args.next() {
            if arg.starts_with("--") {
                // Remove both dashes
                arg.remove(0);
                arg.remove(0);
                if let Some(i) = arg.find('=') {
                    let (lhs, rhs) = arg.split_at(i);
                    if let Some(&mut Value::Opt(Some(ref mut value))) = self.params.get_mut(&Param::Long(lhs.to_owned())) {
                        *value = rhs.to_owned();
                    }
                }
                else {
                    if let Some(&mut Value::Flag(ref mut switch)) = self.params.get_mut(&Param::Long(arg)) {
                        *switch = true;
                    }
                }
            }
            else if arg.starts_with("-") {
                for ch in arg[1..].chars() {
                    match self.params.get_mut(&Param::Short(ch)) {
                        Some(&mut Value::Flag(ref mut switch)) => *switch = true,
                        Some(&mut Value::Opt(ref mut value)) => *value = args.next(),
                        None => (),
                    }
                }
            }
            else {
                self.args.push(arg);
            }
        }
    }

    /// Check if a Flag or Opt has been found after initialization.
    pub fn flagged<P: IntoParam>(&self, name: P) -> bool {
        match self.params.get(&name.into_param()) {
            Some(&Value::Flag(switch)) => switch,
            Some(&Value::Opt(Some(_))) => true,
            _ => false,
        }
    }

    /// Modify the state of a flag. Use `true` if the flag is to be enabled. Use `false` to
    /// disable its use.
    pub fn set_flag<F: IntoParam>(&mut self, flag: F, state: bool) {
        if let Some(&mut Value::Flag(ref mut switch)) = self.params.get_mut(&flag.into_param()) {
            *switch = state;
        }
    }

    /// Modify the state value of an opt. Use `Some(String)` to set if the opt is to be enabled and
    /// has been assigned a value from `String`. Use `None` to disable the opt's use.
    pub fn set_opt<O: IntoParam>(&mut self, opt: O, state: Option<String>) {
        if let Some(&mut Value::Opt(ref mut value)) = self.params.get_mut(&opt.into_param()) {
            *value = state;
        }
    }

    /// Get the state of an Opt. If it has been enabled, it will return a `Some(String)` value
    /// otherwise it will return None.
    pub fn get_opt<O: IntoParam>(&self, opt: O) -> Option<String> {
        if let Some(&Value::Opt(ref value)) = self.params.get(&opt.into_param()) {
            return value.clone();
        }
        None
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
