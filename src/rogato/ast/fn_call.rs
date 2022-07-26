use crate::rogato::db::Value;
use crate::rogato::interpreter::Evaluate;

use super::expression::Expression;
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnCallArgs {
    args: Vec<Expression>,
}

impl FnCallArgs {
    pub fn new(args: Vec<Expression>) -> Self {
        FnCallArgs { args }
    }

    pub fn prepend_arg(&mut self, arg: Expression) {
        self.args.insert(0, arg);
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Expression> {
        self.args.iter()
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

impl Evaluate<Vec<Value>> for FnCallArgs {
    fn evaluate(&self, context: &mut crate::rogato::interpreter::EvalContext) -> Vec<Value> {
        self.iter().map(|a| a.evaluate(context)).collect()
    }
}
