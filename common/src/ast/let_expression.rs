use std::{fmt::Display, rc::Rc};

use crate::util::indent;

use super::{expression::Expression, visitor::Visitor, walker::Walk, ASTDepth, Identifier};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetExpression {
    bindings: LetBindings,
    body: Rc<Expression>,
}

impl LetExpression {
    pub fn new(bindings: LetBindings, body: Rc<Expression>) -> LetExpression {
        LetExpression { bindings, body }
    }

    pub fn bindings(&self) -> &LetBindings {
        &self.bindings
    }

    pub fn body(&self) -> Rc<Expression> {
        Rc::clone(&self.body)
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
    fn walk<V: Visitor<()>>(&self, v: &mut V) {
        v.let_(self);
        self.body.walk(v);
        for (_id, val) in self.bindings.iter() {
            val.walk(v);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetBindings {
    bindings: Vec<(Identifier, Rc<Expression>)>,
}

impl LetBindings {
    pub fn new(bindings: Vec<(Identifier, Rc<Expression>)>) -> LetBindings {
        LetBindings { bindings }
    }

    pub fn from_owned(bindings: Vec<(Identifier, Expression)>) -> LetBindings {
        LetBindings {
            bindings: bindings
                .iter()
                .map(|(id, expr)| (id.clone(), Rc::new(expr.clone())))
                .collect(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<(Identifier, Rc<Expression>)> {
        self.bindings.iter()
    }
}

impl Display for LetBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .bindings
            .iter()
            .map(|(ident, expr)| match &**expr {
                Expression::InlineFnDef(fn_def) => {
                    format!("{}", fn_def)
                }
                _ => {
                    if expr.ast_depth() > 5 {
                        format!("{} =\n{}", ident, indent(expr))
                    } else {
                        format!("{} = {}", ident, expr)
                    }
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
