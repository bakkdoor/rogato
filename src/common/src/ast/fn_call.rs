use crate::eval::{EvalContext, EvalError, Evaluate, ValueRef};

use super::expression::Expression;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnCallArgs {
    args: Vec<Rc<Expression>>,
}

impl FnCallArgs {
    pub fn new(args: Vec<Rc<Expression>>) -> Self {
        FnCallArgs { args }
    }

    pub fn from_owned(args: Vec<Expression>) -> Self {
        FnCallArgs {
            args: args.iter().map(|a| Rc::new(a.clone())).collect(),
        }
    }

    pub fn prepend_arg(&mut self, arg: Rc<Expression>) {
        self.args.insert(0, arg);
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<Expression>> {
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

impl Evaluate<Vec<ValueRef>> for FnCallArgs {
    fn evaluate(&self, context: &mut EvalContext) -> Result<Vec<ValueRef>, EvalError> {
        let mut values = vec![];
        for arg in self.iter() {
            match arg.evaluate(context) {
                Ok(val) => values.push(val),
                Err(e) => return Err(EvalError::FnCallArgumentError(Box::new(e))),
            }
        }
        Ok(values)
    }
}
