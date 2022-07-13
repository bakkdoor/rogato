use super::AST;
use crate::rogato::db::Value;
use crate::rogato::interpreter::{EvalContext, Evaluate};
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program {
    nodes: Vec<AST>,
}

impl Program {
    pub fn new(nodes: Vec<AST>) -> Self {
        Program { nodes }
    }

    #[cfg(test)]
    pub fn from_boxed(nodes: Vec<Box<AST>>) -> Self {
        Program {
            nodes: Vec::from_iter(nodes.iter().map(|d| *d.clone())),
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn iter(&self) -> std::slice::Iter<AST> {
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

impl<'a> Evaluate<'a, Value> for Program {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> Value {
        Value::Array(self.iter().map(|ast| ast.evaluate(context)).collect())
    }
}
