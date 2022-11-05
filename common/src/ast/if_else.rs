use crate::util::indent;

use super::expression::Expression;
use super::visitor::Visitor;
use super::walker::Walk;
use super::ASTDepth;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct IfElse {
    pub condition: Rc<Expression>,
    pub then_expr: Rc<Expression>,
    pub else_expr: Rc<Expression>,
}

impl IfElse {
    pub fn new(
        condition: Rc<Expression>,
        then_expr: Rc<Expression>,
        else_expr: Rc<Expression>,
    ) -> Self {
        Self {
            condition,
            then_expr,
            else_expr,
        }
    }
}

impl Display for IfElse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("if ")?;
        self.condition.fmt(f)?;
        f.write_str(" then\n")?;
        indent(&self.then_expr).fmt(f)?;
        f.write_str("\nelse\n")?;
        indent(&self.else_expr).fmt(f)
    }
}

impl ASTDepth for IfElse {
    fn ast_depth(&self) -> usize {
        1 + self.condition.ast_depth() + self.then_expr.ast_depth() + self.else_expr.ast_depth()
    }
}

impl Walk for IfElse {
    fn walk<V: Visitor<()>>(&self, v: &mut V) {
        v.if_else(self);
        self.condition.walk(v);
        self.then_expr.walk(v);
        self.else_expr.walk(v);
    }
}
