use super::expression::Expression;
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FunctionArgs {
    args: Box<Vec<Expression>>,
}

impl FunctionArgs {
    pub fn new(args: Vec<Expression>) -> Self {
        FunctionArgs {
            args: Box::new(args),
        }
    }
}

impl Display for FunctionArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .args
            .iter()
            .map(|arg| format!("{}", arg))
            .fold(String::from(""), |acc, fmt| format!("{}{}", acc, fmt));

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
