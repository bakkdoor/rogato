use serde_json::Value;

use crate::rogato::db::val;
use crate::rogato::interpreter::Evaluate;

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
    FnCall(Identifier, FnCallArgs),
    OpCall(Identifier, Box<Expression>, Box<Expression>),
    Var(Identifier),
    ConstOrTypeRef(Identifier),
    PropFnRef(Identifier),
    EdgeProp(Box<Expression>, Identifier),
    Let(LetExpression),
    Lambda(Lambda),
    Query(Query),
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

impl<'a> Evaluate<'a, Value> for Expression {
    fn evaluate(&self, context: &mut crate::rogato::interpreter::EvalContext<'a>) -> Value {
        match self {
            Expression::Commented(_c, e) => e.evaluate(context),
            Expression::Lit(lit_exp) => lit_exp.evaluate(context),
            Expression::Sum(a, b) => val::number(
                a.evaluate(context).as_i64().unwrap() + b.evaluate(context).as_i64().unwrap(),
            ),
            Expression::Product(a, b) => val::number(
                a.evaluate(context).as_i64().unwrap() * b.evaluate(context).as_i64().unwrap(),
            ),
            Expression::FnCall(_fn_ident, _args) => todo!("eval fn call"),
            Expression::OpCall(_op_ident, _left, _right) => todo!("eval op call"),
            Expression::Var(id) => match context.env().lookup_var(id) {
                Some(var) => var.evaluate(&mut context.to_owned()),
                None => {
                    eprintln!("Var not found: {}", id);
                    val::null()
                }
            },
            Expression::ConstOrTypeRef(_id) => todo!("eval const or type ref"),
            Expression::PropFnRef(_id) => todo!("eval prop fn ref"),
            Expression::EdgeProp(_id, _edge) => todo!("eval edge prop"),
            Expression::Let(_let_expr) => todo!("eval let expr"),
            Expression::Lambda(_lambda) => todo!("eval lambda"),
            Expression::Query(_query) => todo!("eval query"),
        }
    }
}
