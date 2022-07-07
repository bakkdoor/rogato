use super::expression::Expression;
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnCallArgs {
    args: Box<Vec<Expression>>,
}

impl FnCallArgs {
    pub fn new(args: Vec<Expression>) -> Self {
        FnCallArgs {
            args: Box::new(args),
        }
    }

    pub fn prepend_arg(&mut self, arg: Expression) {
        self.args.insert(0, arg);
    }
}

impl Display for FnCallArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .args
            .iter()
            .map(|arg| format!("{}", arg))
            .fold(String::from(""), |acc, fmt| format!("{} {}", acc, fmt));

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
