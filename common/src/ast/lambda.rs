use std::{fmt::Display, rc::Rc};

use crate::util::indent;

use super::{expression::Expression, walker::Walk, ASTDepth, Identifier};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
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
        f.write_str("(")?;
        self.args.fmt(f)?;

        if self.ast_depth() > 5 {
            f.write_str(" ->\n")?;
            indent(&self.body).fmt(f)?;
        } else {
            f.write_str(" -> ")?;
            self.body.fmt(f)?;
        }

        f.write_str(")")
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

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct LambdaArgs<A: Display + ASTDepth> {
    args: Vec<A>,
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

    pub fn iter(&self) -> std::slice::Iter<A> {
        self.args.iter()
    }

    pub fn get(&self, idx: usize) -> Option<&A> {
        self.args.get(idx)
    }
}

impl<A: Display + ASTDepth> Display for LambdaArgs<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for arg in self.args.iter() {
            if !is_first {
                f.write_str(" ")?;
            }
            arg.fmt(f)?;
            is_first = false;
        }

        Ok(())
    }
}

impl<A: Display + ASTDepth> ASTDepth for LambdaArgs<A> {
    fn ast_depth(&self) -> usize {
        self.args.iter().map(|a| a.ast_depth()).sum::<usize>()
    }
}
