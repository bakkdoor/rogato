pub use super::fn_call::FunctionArgs;
use std::fmt::Display;

pub type Identifier = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    LiteralExpr(LiteralExpr),
    Sum(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    FnCall(Identifier, Box<FunctionArgs>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::LiteralExpr(lit_exp) => lit_exp.fmt(f),
            Expression::Sum(a, b) => f.write_fmt(format_args!("(+ {} {})", a, b)),
            Expression::Product(a, b) => f.write_fmt(format_args!("(* {} {})", a, b)),
            Expression::FnCall(fn_ident, args) => {
                f.write_fmt(format_args!("({} {})", fn_ident, args))
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LiteralExpr {
    Int64(i64),
    String(String),
}

impl Display for LiteralExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralExpr::Int64(num) => f.write_fmt(format_args!("{}", num)),
            LiteralExpr::String(string) => f.write_fmt(format_args!("{}", string)),
        }
    }
}
