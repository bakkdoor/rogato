use super::AST;
use crate::rogato::interpreter::{val, EvalContext, EvalError, Evaluate, ValueRef};
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program {
    nodes: Vec<Rc<AST>>,
}

impl Program {
    pub fn new(nodes: Vec<Rc<AST>>) -> Self {
        Program { nodes }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<AST>> {
        self.nodes.iter()
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str =
            self.iter()
                .map(|ast| format!("{}", ast))
                .fold(String::from(""), |acc, fmt| {
                    if acc.is_empty() {
                        fmt
                    } else {
                        format!("{}\n\n{}", acc, fmt)
                    }
                });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

impl Evaluate<ValueRef> for Program {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut values = vec![];
        for ast in self.iter() {
            values.push(ast.evaluate(context)?)
        }
        Ok(val::list(values))
    }
}
