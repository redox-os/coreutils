use std::borrow::Borrow;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::hash::{Hash,Hasher};
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
/// The parameter styles for short, e.g. `-s`,
/// and for long, e.g. `--long`
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// The Right Hand Side type
struct Rhs<T> {
    /// The RHS value
    value: T,
    /// Counts the number of times a flag/opt has been in use on the command
    occurrences: usize,
}

impl<T> Rhs<T> {
    fn new(value: T) -> Self {
        Rhs { value: value, occurrences: 0 }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// The Value for each parameter
enum Value {
    Flag(Rhs<Rc<RefCell<bool>>>),
    /// The RHS String value is shared between both short and long parameters
    Opt {
        rhs: Rhs<Rc<RefCell<String>>>,
        found: bool,
    },
}

impl Value {
    fn new_opt(value: Rc<RefCell<String>>) -> Self {
        Value::Opt { rhs: Rhs::new(value), found: false }
    }
}

/// Our homebrewed Arg Parser
#[derive(Clone, Debug, Default)]
pub struct ArgParser {
    params: HashMap<Param, Value>,
    invalid: Vec<Param>,
    garbage: (RefCell<bool>, RefCell<String>),
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
            garbage: (RefCell::new(false), RefCell::new(String::with_capacity(0))),
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
    pub fn add_flag(mut self, flags: &[&str]) -> Self {
        let value = Rc::new(RefCell::new(bool::default()));
        for flag in flags.iter() {
            if flag.len() == 1 {
                if let Some(short) = flag.chars().next() {
                    self.params.insert(Param::Short(short), Value::Flag(Rhs::new(value.clone())));
                }
            }
            else if !flag.is_empty() {
                self.params.insert(Param::Long((*flag).to_owned()), Value::Flag(Rhs::new(value.clone())));
            }
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
        let value = Rc::new(RefCell::new("".to_owned()));
        if let Some(short) = short.chars().next() {
            self.params.insert(Param::Short(short), Value::new_opt(value.clone()));
        }
        if !long.is_empty() {
            self.params.insert(Param::Long(long.to_owned()), Value::new_opt(value));
        }
        self
    }

    pub fn add_opt_default(mut self, short: &str, long: &str, default: &str) -> Self {
        let value = Rc::new(RefCell::new(default.to_owned()));
        if let Some(short) = short.chars().next() {
            self.params.insert(Param::Short(short), Value::new_opt(value.clone()));
        }
        if !long.is_empty() {
            self.params.insert(Param::Long(long.to_owned()), Value::new_opt(value));
        }
        self
    }

    /// Start parsing user inputted args for which flags and opts are used at
    /// runtime. The rest of the args that are not associated to opts get added
    /// to `ArgParser.args`.
    pub fn parse<A: Iterator<Item=String>>(&mut self, args: A) {
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
                        Some(&mut Value::Opt { rhs: ref mut opt_rhs, ref mut found }) => {
                            if (*opt_rhs.value).borrow().is_empty() {
                                opt_rhs.occurrences = 1;
                            }
                            else {
                                opt_rhs.occurrences += 1;
                            }
                            (*opt_rhs.value).borrow_mut().clear();
                            (*opt_rhs.value).borrow_mut().push_str(rhs);
                            *found = true;
                        }
                        _ => self.invalid.push(Param::Long(lhs.to_owned())),
                    }
                }
                else {
                    match self.params.get_mut(arg) {
                        Some(&mut Value::Flag(ref mut rhs)) => {
                            *(*rhs.value).borrow_mut() = true;
                            rhs.occurrences += 1;
                        }
                        Some(&mut Value::Opt { ref mut rhs, ref mut found }) => {
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
                            *(*rhs.value).borrow_mut() = true;
                            rhs.occurrences += 1;
                        }
                        Some(&mut Value::Opt { ref mut rhs, ref mut found }) => {
                            let rest: String = chars.collect();
                            if !rest.is_empty() {
                                *(*rhs.value).borrow_mut() = rest;
                                *found = true;
                            } else {
                                *(*rhs.value).borrow_mut() = args.next().map(|a| {*found = true; a}).unwrap_or("".to_owned());
                            }
                            break;
                        }
                        None => self.invalid.push(Param::Short(ch)),
                    }
                }
            }
            else if arg.contains("=") {
                if arg.is_empty() {
                    //Arg `--` means we are done parsing args, collect the rest
                    self.args.extend(args);
                    break;
                }
                if let Some(i) = arg.find('=') {
                    let (lhs, rhs) = arg.split_at(i);
                    let rhs = &rhs[1..]; // slice off the `=` char
                    match self.params.get_mut(lhs) {
                        Some(&mut Value::Opt { rhs: ref mut opt_rhs, ref mut found }) => {
                            if (*opt_rhs.value).borrow().is_empty() {
                                opt_rhs.occurrences = 1;
                            }
                            else {
                                opt_rhs.occurrences += 1;
                            }
                            (*opt_rhs.value).borrow_mut().clear();
                            (*opt_rhs.value).borrow_mut().push_str(rhs);
                            *found = true;
                        }
                        _ => self.invalid.push(Param::Long(lhs.to_owned())),
                    }
                }
            }
            else {
                self.args.push(arg);
            }
        }
    }

    /// Get the number of times a flag or opt has been found after parsing.
    pub fn count<P: Hash + Eq + ?Sized>(&self, name: &P) -> usize
        where Param: Borrow<P>
    {
        match self.params.get(name) {
            Some(&Value::Flag(ref rhs)) => rhs.occurrences,
            Some(&Value::Opt { ref rhs, .. }) => rhs.occurrences,
            _ => 0,
        }
    }

    /// Check if a flag or opt has been found after initialization.
    pub fn found<P: Hash + Eq + ?Sized>(&self, name: &P) -> bool
        where Param: Borrow<P>
    {
        match self.params.get(name) {
            Some(&Value::Flag(ref rhs)) => *(*rhs.value).borrow_mut(),
            Some(&Value::Opt { found, .. }) => found,
            _ => false,
        }
    }

    /// Modify the state of a flag. Use `true` if the flag is to be enabled. Use `false` to
    /// disable its use.
    pub fn flag<F: Hash + Eq + ?Sized>(&mut self, flag: &F) -> RefMut<bool>
        where Param: Borrow<F>
    {
        if let Some(&mut Value::Flag(ref mut rhs)) = self.params.get_mut(flag) {
            return (*rhs.value).borrow_mut();
        }
        self.garbage.0.borrow_mut()
    }

    /// Modify the state value of an opt. Use `Some(String)` to set if the opt is to be enabled and
    /// has been assigned a value from `String`. Use `None` to disable the opt's use.
    pub fn opt<O: Hash + Eq + ?Sized>(&mut self, opt: &O) -> RefMut<String>
        where Param: Borrow<O>
    {
        if let Some(&mut Value::Opt { ref mut rhs, .. }) = self.params.get_mut(opt) {
            return (*rhs.value).borrow_mut();
        }
        self.garbage.1.borrow_mut()
    }

    /// Get the value of an Opt. If it has been set or defaulted, it will return a `Some(String)`
    /// value otherwise it will return None.
    pub fn get_opt<O: Hash + Eq + ?Sized>(&self, opt: &O) -> Option<String>
        where Param: Borrow<O>
    {
        if let Some(&Value::Opt { ref rhs, .. }) = self.params.get(opt) {
            return Some((*rhs.value).borrow().clone());
        }
        None
    }

    pub fn found_invalid(&self) -> Result<(), String> {
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
        parser = parser.add_flag(&["a"])
                       .add_flag(&["v"]);
        parser.parse(args.into_iter());
        assert!(parser.found(&'a'));
        assert!(!parser.found(&'v'));
        assert!(parser.args[0] == "-v");
    }

    #[test]
    fn short_opts() {
        let args = vec![String::from("binname"), String::from("-asdf"), String::from("-f"), String::from("foo")];
        let mut parser = ArgParser::new(4);
        parser = parser.add_flag(&["a"])
                       .add_flag(&["d"])
                       .add_opt("s", "")
                       .add_opt("f", "");
        parser.parse(args.into_iter());
        assert!(parser.found(&'a'));
        assert!(!parser.found(&'d'));
        assert!(parser.get_opt(&'s') == Some(String::from("df")));
        assert!(parser.get_opt(&'f') == Some(String::from("foo")));
    }

    #[test]
    fn long_opts() {
        let args = vec![String::from("binname"), String::from("--foo=bar")];
        let mut parser = ArgParser::new(4);
        parser = parser.add_opt("", "foo");
        parser.parse(args.into_iter());
        assert!(parser.get_opt("foo") == Some(String::from("bar")));
    }

    #[test]
    fn dd_style_opts() {
        let args = vec![String::from("binname"), String::from("-h"), String::from("if=bar"), String::from("of=foo")];
        let mut parser = ArgParser::new(4);
        parser = parser.add_flag(&["h"])
                       .add_opt("", "if")
                       .add_opt("", "of");
        parser.parse(args.into_iter());
        assert!(parser.found(&'h'));
        assert!(parser.get_opt("if") == Some(String::from("bar")));
        assert!(parser.get_opt("of") == Some(String::from("foo")));
    }
}
