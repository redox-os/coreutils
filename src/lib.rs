#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Param {
    Short(char),
    Long(String),
}

use std::borrow::Borrow;

impl Borrow<str> for Param {
    fn borrow(&self) -> &str {
        if let Param::Long(ref string) = *self {
            string
        } else {
            ""
        }
    }
}

impl Borrow<char> for Param {
    fn borrow(&self) -> &char {
        if let Param::Short(ref ch) = *self {
            ch
        } else {
            const CH: &'static char = &'\0';
            CH
        }
    }
}

use std::hash::{Hash,Hasher};

impl Hash for Param {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Param::Short(ref c) => c.hash(state),
            Param::Long(ref s) => s.hash(state)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Value {
    Flag(bool),
    Opt(Option<String>),
}

use std::collections::HashMap;

/// Our homebrewed Arg Parser
#[derive(Clone, Debug, Default)]
pub struct ArgParser {
    params: HashMap<Param, Value>,
    invalid: Vec<Param>,
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
            invalid: Vec::new(),
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
        while let Some(arg) = args.next() {
            if arg.starts_with("--") {
                // Remove both dashes
                let arg = &arg[2..];
                if arg.len() == 0 {
                    //Arg `--` means we are done parsing args, collect the rest
                    self.args.extend(args);
                    break;
                }
                if let Some(i) = arg.find('=') {
                    let (lhs, rhs) = arg.split_at(i);
                    match self.params.get_mut(lhs) {
                        // slice off the `=` char
                        Some(&mut Value::Opt(ref mut value)) => *value = Some(rhs[1..].to_owned()),
                        _ => self.invalid.push(Param::Long(lhs.to_owned())),
                    }
                }
                else {
                    match self.params.get_mut(arg) {
                        Some(&mut Value::Flag(ref mut switch)) => *switch = true,
                        _ => self.invalid.push(Param::Long(arg.to_owned())),
                    }
                }
            }
            else if arg.starts_with("-") {
                let mut chars = arg[1..].chars();
                while let Some(ch) = chars.next() {
                    match self.params.get_mut(&ch) {
                        Some(&mut Value::Flag(ref mut switch)) => *switch = true,
                        Some(&mut Value::Opt(ref mut value)) => {
                            let rest: String = chars.collect();
                            if rest.len() > 0 {
                                *value = Some(rest);
                            } else {
                                *value = args.next()
                            }
                            break;
                        },
                        None => self.invalid.push(Param::Short(ch)),
                    }
                }
            }
            else {
                self.args.push(arg);
            }
        }
    }

    /// Check if a Flag or Opt has been found after initialization.
    pub fn flagged<P: Hash + Eq + ?Sized>(&self, name: &P) -> bool
        where Param: Borrow<P>
    {
        match self.params.get(name) {
            Some(&Value::Flag(switch)) => switch,
            Some(&Value::Opt(Some(_))) => true,
            _ => false,
        }
    }

    /// Modify the state of a flag. Use `true` if the flag is to be enabled. Use `false` to
    /// disable its use.
    pub fn set_flag<F: Hash + Eq + ?Sized>(&mut self, flag: &F, state: bool)
        where Param: Borrow<F>
    {
        if let Some(&mut Value::Flag(ref mut switch)) = self.params.get_mut(flag) {
            *switch = state;
        }
    }

    /// Modify the state value of an opt. Use `Some(String)` to set if the opt is to be enabled and
    /// has been assigned a value from `String`. Use `None` to disable the opt's use.
    pub fn set_opt<O: Hash + Eq + ?Sized>(&mut self, opt: &O, state: Option<String>)
        where Param: Borrow<O>
    {
        if let Some(&mut Value::Opt(ref mut value)) = self.params.get_mut(opt) {
            *value = state;
        }
    }

    /// Get the state of an Opt. If it has been enabled, it will return a `Some(String)` value
    /// otherwise it will return None.
    pub fn get_opt<O: Hash + Eq + ?Sized>(&self, opt: &O) -> Option<String>
        where Param: Borrow<O>
    {
        if let Some(&Value::Opt(ref value)) = self.params.get(opt) {
            return value.clone();
        }
        None
    }

    pub fn flagged_invalid(&self) -> Result<(), String> {
        if self.invalid.is_empty() {
            return Ok(());
        }

        let mut and: bool = false;
        let mut output =
            if self.invalid.len() == 1 {
                "Invalid parameter"
            } else {
                and = true;
                "Invalid parameters"
            }.to_owned();

        let mut iter = self.invalid.iter().peekable();
        while let Some(param) = iter.next() {
            match param {
                &Param::Short(ch) => {
                    output += " '-";
                    output.push(ch);
                    output.push('\'');
                }
                &Param::Long(ref s) => {
                    output += " '--";
                    output += s;
                    output.push('\'');
                }
            }
            if and && iter.peek().is_some() {
                output += " and"
            }
        }
        output.push('\n');
        Err(output)
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
