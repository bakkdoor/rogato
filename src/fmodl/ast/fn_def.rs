use super::Identifier;
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnDefArgs {
    args: Box<Vec<Identifier>>,
}

impl FnDefArgs {
    pub fn new(args: Vec<Identifier>) -> Self {
        FnDefArgs {
            args: Box::new(args),
        }
    }
}

impl Display for FnDefArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .args
            .iter()
            .map(|arg| format!("{}", arg))
            .fold(String::from(""), |acc, fmt| format!("{} {}", acc, fmt));

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
