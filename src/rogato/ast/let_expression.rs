use std::fmt::Display;

use crate::rogato::util::indent;

use super::{expression::Expression, visitor::Visitor, walker::Walk, Identifier};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetExpression {
    bindings: LetBindings,
    body: Box<Expression>,
}

impl LetExpression {
    pub fn new(bindings: LetBindings, body: Box<Expression>) -> LetExpression {
        LetExpression {
            bindings: bindings,
            body: body,
        }
    }
}

impl Display for LetExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "let\n{}\nin\n{}",
            indent(self.bindings.clone()),
            indent(self.body.clone())
        ))
    }
}

impl Walk for LetExpression {
    fn walk<V: Visitor>(&self, v: &mut V) {
        v.let_(self);
        self.body.walk(v);
        for (_id, val) in self.bindings.iter() {
            val.walk(v);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetBindings {
    bindings: Vec<(Identifier, Expression)>,
}

impl LetBindings {
    pub fn new(bindings: Vec<(Identifier, Expression)>) -> LetBindings {
        LetBindings { bindings: bindings }
    }

    pub fn iter(&self) -> std::slice::Iter<(String, Expression)> {
        self.bindings.iter()
    }
}

impl Display for LetBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .bindings
            .iter()
            .map(|(ident, expr)| format!("{} =\n{}", ident, indent(expr)))
            .fold(String::from(""), |acc, fmt| {
                if acc == "" {
                    fmt
                } else {
                    format!("{}\n\n{}", acc, fmt)
                }
            });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
