pub use super::fn_call::FunctionArgs;
use std::fmt::Display;

pub type Identifier = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Literal(Literal),
    Sum(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    FnCall(Identifier, Box<FunctionArgs>),
    Var(Identifier),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(lit_exp) => lit_exp.fmt(f),
            Expression::Sum(a, b) => f.write_fmt(format_args!("(+ {} {})", a, b)),
            Expression::Product(a, b) => f.write_fmt(format_args!("(* {} {})", a, b)),
            Expression::FnCall(fn_ident, args) => {
                f.write_fmt(format_args!("({}{})", fn_ident, args))
            }
            Expression::Var(id) => f.write_str(id),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Literal {
    Int64Lit(i64),
    StringLit(String),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int64Lit(num) => f.write_fmt(format_args!("{}", num)),
            Literal::StringLit(string) => f.write_fmt(format_args!("{}", string)),
        }
    }
}
