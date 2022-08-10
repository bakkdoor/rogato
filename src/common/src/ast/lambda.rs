use std::{fmt::Display, rc::Rc};

use smol_str::SmolStr;

use crate::util::indent;

use super::{expression::Expression, walker::Walk, ASTDepth, Identifier};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Lambda {
    args: LambdaArgs<Identifier>,
    body: Rc<Expression>,
}

impl Lambda {
    pub fn new(args: LambdaArgs<Identifier>, body: Rc<Expression>) -> Lambda {
        Lambda { args, body }
    }
    pub fn get_arg(&self, i: usize) -> Option<&Identifier> {
        self.args.get(i)
    }

    pub fn args(&self) -> &LambdaArgs<Identifier> {
        &self.args
    }

    pub fn arg_count(&self) -> usize {
        self.args.len()
    }

    pub fn body(&self) -> Rc<Expression> {
        Rc::clone(&self.body)
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
    fn walk<V: super::visitor::Visitor<()>>(&self, v: &mut V) {
        v.lambda(self);
        self.body.walk(v);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LambdaArgs<A: Display + ASTDepth> {
    args: Vec<A>,
}

impl ASTDepth for SmolStr {
    fn ast_depth(&self) -> usize {
        1
    }
}

impl<A: Display + ASTDepth> LambdaArgs<A> {
    pub fn new(args: Vec<A>) -> LambdaArgs<A> {
        LambdaArgs { args }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::slice::Iter<A> {
        self.args.iter()
    }

    pub fn get(&self, idx: usize) -> Option<&A> {
        self.args.get(idx)
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
