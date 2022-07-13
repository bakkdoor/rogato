use std::fmt::Display;

use serde_json::Value;

use crate::rogato::{
    db::val,
    interpreter::{EvalContext, Evaluate},
    util::indent,
};

use super::{expression::Expression, walker::Walk, ASTDepth, Identifier};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Lambda {
    args: LambdaArgs<Identifier>,
    body: Box<Expression>,
}

impl Lambda {
    pub fn new(args: LambdaArgs<Identifier>, body: Box<Expression>) -> Lambda {
        Lambda { args, body }
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ast_depth() > 5 {
            f.write_fmt(format_args!(
                "({} ->\n{})",
                self.args,
                indent(self.body.clone())
            ))
        } else {
            f.write_fmt(format_args!("({} -> {})", self.args, self.body))
        }
    }
}

impl ASTDepth for Lambda {
    fn ast_depth(&self) -> usize {
        1 + self.args.len() + self.body.ast_depth()
    }
}

impl Walk for Lambda {
    fn walk<V: super::visitor::Visitor>(&self, v: &mut V) {
        v.lambda(self);
        self.body.walk(v);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LambdaArgs<A: Display> {
    args: Vec<A>,
}

impl ASTDepth for String {
    fn ast_depth(&self) -> usize {
        1
    }
}

impl<A: Display + ASTDepth> LambdaArgs<A> {
    pub fn new(args: Vec<A>) -> LambdaArgs<A> {
        LambdaArgs { args }
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

impl<A: Display + ASTDepth> Display for LambdaArgs<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self.args.iter().fold(String::from(""), |acc, arg| {
            if acc.is_empty() {
                arg.to_string()
            } else {
                format!("{} {}", acc, arg)
            }
        });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

impl<A: Display + ASTDepth> ASTDepth for LambdaArgs<A> {
    fn ast_depth(&self) -> usize {
        self.args.iter().map(|a| a.ast_depth()).sum::<usize>()
    }
}

impl<'a, A: Display + ASTDepth + Evaluate<'a, Value>> Evaluate<'a, Value> for LambdaArgs<A> {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> Value {
        let mut vec = Vec::new();
        for arg in self.args.iter() {
            vec.push(arg.evaluate(context))
        }
        val::array(vec)
    }
}
