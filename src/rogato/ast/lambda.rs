use std::{fmt::Display, rc::Rc};

use crate::rogato::{
    db::{val, ValueRef},
    interpreter::{EvalContext, EvalError, Evaluate},
    util::indent,
};

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

impl Evaluate<ValueRef> for Lambda {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let evaluated_args: Vec<ValueRef> = self.args.evaluate(context)?;
        let mut fn_context = context.with_child_env();
        let given_args = evaluated_args.len();
        let expected_args = self.args.len();

        if given_args != expected_args {
            panic!(
                "Mismatch in lambda arguments. Expected {} but got {}",
                expected_args, given_args
            )
        }

        for (i, arg_val) in evaluated_args.iter().enumerate().take(self.args.len()) {
            let arg_name = self.args.get(i).unwrap().clone();
            fn_context.define_var(&arg_name, arg_val.clone())
        }
        self.body.evaluate(&mut fn_context)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LambdaArgs<A: Display + ASTDepth> {
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

impl<A: Display + ASTDepth + Evaluate<ValueRef>> Evaluate<ValueRef> for LambdaArgs<A> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut vec = Vec::new();
        for arg in self.args.iter() {
            vec.push(arg.evaluate(context)?)
        }
        Ok(val::list(vec))
    }
}

impl<A: Display + ASTDepth + Evaluate<ValueRef>> Evaluate<Vec<ValueRef>> for LambdaArgs<A> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<Vec<ValueRef>, EvalError> {
        let mut vec = Vec::new();
        for arg in self.args.iter() {
            vec.push(arg.evaluate(context)?)
        }
        Ok(vec)
    }
}
