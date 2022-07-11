use std::fmt::Display;

use super::{expression::Expression, Identifier};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Lambda {
    args: LambdaArgs<Identifier>,
    body: Box<Expression>,
}

impl Lambda {
    pub fn new(args: LambdaArgs<Identifier>, body: Box<Expression>) -> Lambda {
        Lambda {
            args: args,
            body: body,
        }
    }

    pub fn body<'a>(&'a self) -> &'a Expression {
        &self.body
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({} -> {})", self.args, self.body))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LambdaArgs<A: Display> {
    args: Vec<A>,
}

impl<A: Display> LambdaArgs<A> {
    pub fn new(args: Vec<A>) -> LambdaArgs<A> {
        LambdaArgs { args: args }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.args.len()
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::slice::Iter<A> {
        self.args.iter()
    }
}

impl<A: Display> Display for LambdaArgs<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self.args.iter().fold(String::from(""), |acc, arg| {
            if acc == "" {
                arg.to_string()
            } else {
                format!("{} {}", acc, arg)
            }
        });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
