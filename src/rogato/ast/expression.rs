pub use super::fn_call::FnCallArgs;
pub use super::fn_def::FnDefArgs;
pub use super::lambda::{Lambda, LambdaArgs};
pub use super::let_expression::{LetBindings, LetExpression};
pub use super::literal::*;
pub use super::query::{Query, QueryBinding, QueryBindings, QueryGuards};
use super::Identifier;
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Commented(String, Box<Expression>),
    Lit(Literal),
    Sum(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    FnCall(Identifier, Box<FnCallArgs>),
    OpCall(Identifier, Box<Expression>, Box<Expression>),
    Var(Identifier),
    ConstOrTypeRef(Identifier),
    PropFnRef(Identifier),
    EdgeProp(Box<Expression>, Identifier),
    Let(Box<LetExpression>),
    Lambda(Box<Lambda>),
    Query(Box<Query>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Commented(comment, exp) => {
                f.write_fmt(format_args!("//{}\n{}", comment, exp))
            }
            Expression::Lit(lit_exp) => lit_exp.fmt(f),
            Expression::Sum(a, b) => f.write_fmt(format_args!("({} + {})", a, b)),
            Expression::Product(a, b) => f.write_fmt(format_args!("({} * {})", a, b)),
            Expression::FnCall(fn_ident, args) => {
                f.write_fmt(format_args!("({}{})", fn_ident, args))
            }
            Expression::OpCall(op_ident, left, right) => {
                f.write_fmt(format_args!("({} {} {})", left, op_ident, right))
            }
            Expression::Var(id) => f.write_str(id),
            Expression::ConstOrTypeRef(id) => f.write_str(id),
            Expression::PropFnRef(id) => f.write_fmt(format_args!(".{}", id)),
            Expression::EdgeProp(id, edge) => f.write_fmt(format_args!("{}#{}", id, edge)),
            Expression::Let(let_expr) => let_expr.fmt(f),
            Expression::Lambda(lambda) => lambda.fmt(f),
            Expression::Query(query) => query.fmt(f),
        }
    }
}
