use crate::rogato::db::{val, Value};
use crate::rogato::interpreter::{EvalContext, Evaluate};

pub use super::fn_call::FnCallArgs;
pub use super::fn_def::FnDefArgs;
pub use super::lambda::{Lambda, LambdaArgs};
pub use super::let_expression::{LetBindings, LetExpression};
pub use super::literal::*;
pub use super::query::{Query, QueryBinding, QueryBindings, QueryGuards};
use super::{ASTDepth, Identifier};
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
    Symbol(Identifier),
    Quoted(Box<Expression>),
    Unquoted(Box<Expression>),
}

impl ASTDepth for Expression {
    fn ast_depth(&self) -> usize {
        match self {
            Expression::Commented(_, e) => 1 + e.ast_depth(),
            Expression::Lit(lit_exp) => lit_exp.ast_depth(),
            Expression::Sum(a, b) => a.ast_depth() + b.ast_depth(),
            Expression::Product(a, b) => a.ast_depth() + b.ast_depth(),
            Expression::FnCall(_id, args) => args.iter().map(|a| a.ast_depth()).sum(),
            Expression::OpCall(_id, left, right) => left.ast_depth() + right.ast_depth(),
            Expression::Var(_id) => 1,
            Expression::ConstOrTypeRef(_id) => 1,
            Expression::PropFnRef(_id) => 1,
            Expression::EdgeProp(expr, _edge) => 1 + expr.ast_depth(),
            Expression::Let(let_expr) => let_expr.ast_depth(),
            Expression::Lambda(lambda) => lambda.ast_depth(),
            Expression::Query(query) => query.ast_depth(),
            Expression::Symbol(_id) => 1,
            Expression::Quoted(expr) => 1 + expr.ast_depth(),
            Expression::Unquoted(expr) => 1 + expr.ast_depth(),
        }
    }
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
            Expression::Symbol(id) => f.write_fmt(format_args!("^{}", id)),
            Expression::Quoted(expr) => {
                let expr_fmt = format!("{}", expr);
                if expr_fmt.starts_with('(') && expr_fmt.ends_with(')') {
                    f.write_fmt(format_args!("^{}", expr_fmt))
                } else {
                    f.write_fmt(format_args!("^({})", expr_fmt))
                }
            }
            Expression::Unquoted(expr) => {
                let expr_fmt = format!("{}", expr);
                if expr_fmt.starts_with('(') && expr_fmt.ends_with(')') {
                    f.write_fmt(format_args!("~{}", expr_fmt))
                } else {
                    f.write_fmt(format_args!("~({})", expr_fmt))
                }
            }
        }
    }
}

impl<'a> Evaluate<'a, Value> for Expression {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> Value {
        match self {
            Expression::Commented(_c, e) => e.evaluate(context),
            Expression::Lit(lit_exp) => lit_exp.evaluate(context),
            Expression::Sum(a, b) => {
                match (a.evaluate(context), b.evaluate(context)) {
                    (Value::Number(n1), Value::Number(n2)) => {
                        // todo: add support for other number types
                        let num = n1.as_i64().unwrap() + n2.as_i64().unwrap();
                        val::number(num)
                    }
                    (val1, val2) => context.call_function("+", vec![val1, val2]),
                }
            }
            Expression::Product(a, b) => val::number(
                a.evaluate(context).as_i64().unwrap() * b.evaluate(context).as_i64().unwrap(),
            ),
            Expression::FnCall(fn_ident, args) => {
                let call_args = args.iter().map(|a| a.evaluate(context)).collect();
                context.call_function(fn_ident, call_args)
            }
            Expression::OpCall(op_ident, left, right) => {
                val::string(format!("{} {} {}", op_ident, left, right))
            }
            Expression::Var(id) => match context.lookup_var(id) {
                Some(var) => var.to_owned(),
                None => {
                    eprintln!("Var not found: {}", id);
                    val::null()
                }
            },
            Expression::ConstOrTypeRef(_id) => val::string("eval const or type ref"),
            Expression::PropFnRef(_id) => val::string("eval prop fn ref"),
            Expression::EdgeProp(_id, _edge) => val::string("eval edge prop"),
            Expression::Let(let_expr) => let_expr.evaluate(context),
            Expression::Lambda(_lambda) => val::string("eval lambda"),
            Expression::Query(_query) => val::string("eval query"),
            Expression::Symbol(id) => val::string(format!("Symbol ^{}", id)), // likely need custom value types besides just json values to properly support symbols
            Expression::Quoted(expr) => val::string(format!("^({})", expr)),
            Expression::Unquoted(expr) => val::string(format!("~({})", expr)),
        }
    }
}
