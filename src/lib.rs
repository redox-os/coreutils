use std::collections::HashMap;
use std::borrow::Borrow;
use std::hash::{Hash,Hasher};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Param {
    Short(char),
    Long(String),
}

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

impl Hash for Param {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Param::Short(ref c) => c.hash(state),
            Param::Long(ref s) => s.hash(state)
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
struct Rhs<T> {
    value: T,
    occurrences: usize,
}

impl<T> Rhs<T> {
    fn new<U: Into<T>>(value: U) -> Self {
        Rhs { value: value.into(), occurrences: 0 }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum OptRhs {
    With(Rhs<String>, bool),
    Empty,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Value {
    Flag(Rhs<bool>),
    Opt(OptRhs),
}

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
            self.params.insert(Param::Short(short), Value::Flag(Rhs::default()));
        }
        if !long.is_empty() {
            self.params.insert(Param::Long(long.to_owned()), Value::Flag(Rhs::default()));
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
            self.params.insert(Param::Short(short), Value::Opt(OptRhs::Empty));
        }
        if !long.is_empty() {
            self.params.insert(Param::Long(long.to_owned()), Value::Opt(OptRhs::Empty));
        }
        self
    }

    pub fn add_opt_default(mut self, short: &str, long: &str, default: &str) -> Self {
        if let Some(short) = short.chars().next() {
            self.params.insert(Param::Short(short), Value::Opt(OptRhs::With(Rhs::new(default), false)));
        }
        if !long.is_empty() {
            self.params.insert(Param::Long(long.to_owned()), Value::Opt(OptRhs::With(Rhs::new(default), false)));
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
                if arg.is_empty() {
                    //Arg `--` means we are done parsing args, collect the rest
                    self.args.extend(args);
                    break;
                }
                if let Some(i) = arg.find('=') {
                    let (lhs, rhs) = arg.split_at(i);
                    let rhs = &rhs[1..]; // slice off the `=` char
                    match self.params.get_mut(lhs) {
                        Some(&mut Value::Opt(ref mut value)) => {
                            match value {
                                &mut OptRhs::With(ref mut opt_rhs, ref mut found) => {
                                    opt_rhs.value.clear();
                                    opt_rhs.value.push_str(rhs);
                                    opt_rhs.occurrences += 1;
                                    *found = true;
                                }
                                &mut OptRhs::Empty => {
                                    let mut rhs = Rhs::new(rhs);
                                    rhs.occurrences = 1;
                                    *value = OptRhs::With(rhs, true);
                                }
                            }
                        }
                        _ => self.invalid.push(Param::Long(lhs.to_owned())),
                    }
                }
                else {
                    match self.params.get_mut(arg) {
                        Some(&mut Value::Flag(ref mut rhs)) => {
                            rhs.value = true;
                            rhs.occurrences += 1;
                        }
                        Some(&mut Value::Opt(OptRhs::With(ref mut rhs, ref mut found))) => {
                            rhs.occurrences += 1;
                            *found = true;
                        }
                        _ => self.invalid.push(Param::Long(arg.to_owned())),
                    }
                }
            }
            else if arg.starts_with("-") {
                let mut chars = arg[1..].chars();
                while let Some(ch) = chars.next() {
                    match self.params.get_mut(&ch) {
                        Some(&mut Value::Flag(ref mut rhs)) => {
                            rhs.value = true;
                            rhs.occurrences += 1;
                        }
                        Some(&mut Value::Opt(ref mut opt_rhs)) => {
                            let rest: String = chars.collect();
                            if !rest.is_empty() {
                                *opt_rhs = OptRhs::With(Rhs::new(rest), true);
                            } else {
                                *opt_rhs = args.next().map(|a| OptRhs::With(Rhs::new(a), true)).unwrap_or(OptRhs::Empty);
                            }
                            break;
                        }
                        None => self.invalid.push(Param::Short(ch)),
                    }
                }
            }
            else {
                self.args.push(arg);
            }
        }
    }

    /// Check the number of time a Flag or Opt has been found after initialization.
    pub fn count<P: Hash + Eq + ?Sized>(&self, name: &P) -> usize
        where Param: Borrow<P>
    {
        match self.params.get(name) {
            Some(&Value::Flag(ref rhs)) => rhs.occurrences,
            Some(&Value::Opt(OptRhs::With(ref rhs, _))) => rhs.occurrences,
            _ => 0,
        }
    }

    /// Check if a Flag or Opt has been found after initialization.
    pub fn flagged<P: Hash + Eq + ?Sized>(&self, name: &P) -> bool
        where Param: Borrow<P>
    {
        match self.params.get(name) {
            Some(&Value::Flag(ref rhs)) => rhs.value,
            Some(&Value::Opt(OptRhs::With(_, found))) => found,
            _ => false,
        }
    }

    /// Modify the state of a flag. Use `true` if the flag is to be enabled. Use `false` to
    /// disable its use.
    pub fn set_flag<F: Hash + Eq + ?Sized>(&mut self, flag: &F, state: bool)
        where Param: Borrow<F>
    {
        if let Some(&mut Value::Flag(ref mut rhs)) = self.params.get_mut(flag) {
            rhs.value = state;
        }
    }

    /// Modify the state value of an opt. Use `Some(String)` to set if the opt is to be enabled and
    /// has been assigned a value from `String`. Use `None` to disable the opt's use.
    pub fn set_opt<O: Hash + Eq + ?Sized>(&mut self, opt: &O, state: Option<String>)
        where Param: Borrow<O>
    {
        if let Some(&mut Value::Opt(OptRhs::With(ref mut rhs, ref mut found))) = self.params.get_mut(opt) {
            match state {
                Some(input) => {
                    rhs.value = input;
                    *found = true;
                }
                None => *found = false,
            }
        }
    }

    /// Get the value of an Opt. If it has been set or defaulted, it will return a `Some(String)`
    /// value otherwise it will return None.
    pub fn get_opt<O: Hash + Eq + ?Sized>(&self, opt: &O) -> Option<String>
        where Param: Borrow<O>
    {
        if let Some(&Value::Opt(OptRhs::With(ref rhs, _))) = self.params.get(opt) {
            return Some(rhs.value.clone());
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
                output += " and";
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

#[cfg(test)]
mod tests {
    use super::ArgParser;

    #[test]
    fn stop_parsing() {
        let args = vec![String::from("binname"), String::from("-a"), String::from("--"), String::from("-v")];
        let mut parser = ArgParser::new(2);
        parser = parser.add_flag("a", "")
                       .add_flag("v", "");
        parser.initialize(args.into_iter());
        assert!(parser.flagged(&'a'));
        assert!(!parser.flagged(&'v'));
        assert!(parser.args[0] == "-v");
    }

    #[test]
    fn short_opts() {
        let args = vec![String::from("binname"), String::from("-asdf"), String::from("-f"), String::from("foo")];
        let mut parser = ArgParser::new(4);
        parser = parser.add_flag("a", "")
                       .add_flag("d", "")
                       .add_opt("s", "")
                       .add_opt("f", "");
        parser.initialize(args.into_iter());
        assert!(parser.flagged(&'a'));
        assert!(!parser.flagged(&'d'));
        assert!(parser.get_opt(&'s') == Some(String::from("df")));
        assert!(parser.get_opt(&'f') == Some(String::from("foo")));
    }

    #[test]
    fn long_opts() {
        let args = vec![String::from("binname"), String::from("--foo=bar")];
        let mut parser = ArgParser::new(4);
        parser = parser.add_opt("", "foo");
        parser.initialize(args.into_iter());
        assert!(parser.get_opt("foo") == Some(String::from("bar")));
    }
}
