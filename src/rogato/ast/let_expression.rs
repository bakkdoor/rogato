use std::fmt::Display;

use crate::rogato::{
    db::Value,
    interpreter::{EvalContext, Evaluate},
    util::indent,
};

use super::{expression::Expression, visitor::Visitor, walker::Walk, ASTDepth, Identifier};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetExpression {
    bindings: LetBindings,
    body: Box<Expression>,
}

impl LetExpression {
    pub fn new(bindings: LetBindings, body: Box<Expression>) -> LetExpression {
        LetExpression { bindings, body }
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

impl ASTDepth for LetExpression {
    fn ast_depth(&self) -> usize {
        1 + self.bindings.ast_depth() + self.body.ast_depth()
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

impl Evaluate<Value> for LetExpression {
    fn evaluate(&self, context: &mut EvalContext) -> Value {
        for (id, expr) in self.bindings.iter() {
            let val = expr.evaluate(context);
            context.define_var(id, val)
        }
        self.body.evaluate(context)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetBindings {
    bindings: Vec<(Identifier, Expression)>,
}

impl LetBindings {
    pub fn new(bindings: Vec<(Identifier, Expression)>) -> LetBindings {
        LetBindings { bindings }
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
            .map(|(ident, expr)| {
                if expr.ast_depth() > 5 {
                    format!("{} =\n{}", ident, indent(expr))
                } else {
                    format!("{} = {}", ident, expr)
                }
            })
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

impl ASTDepth for LetBindings {
    fn ast_depth(&self) -> usize {
        1 + self
            .bindings
            .iter()
            .map(|(_id, expr)| expr.ast_depth())
            .sum::<usize>()
    }
}
