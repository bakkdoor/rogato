use super::expression::{Expression, Identifier};
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnDefBody {
    expressions: Box<Vec<Expression>>,
}

impl FnDefBody {
    pub fn new(expressions: Vec<Expression>) -> FnDefBody {
        FnDefBody {
            expressions: Box::new(expressions),
        }
    }
}

impl Display for FnDefBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .expressions
            .iter()
            .map(|exp| format!("{}", exp))
            .fold(String::from(""), |acc, fmt| format!("{} {}", acc, fmt));

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
