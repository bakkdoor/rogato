use indent_write::indentable::Indentable;

pub use super::fn_call::FnCallArgs;
pub use super::fn_def::FnDefArgs;
use super::Identifier;
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Lit(Literal),
    Sum(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    FnCall(Identifier, Box<FnCallArgs>),
    OpCall(Identifier, Box<Expression>, Box<Expression>),
    Var(Identifier),
    Let(Box<LetBindings>, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
            Expression::Let(bindings, body) => f.write_fmt(format_args!(
                "let\n{}\nin\n{}",
                bindings.indented("    "),
                body.indented("    ")
            )),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetBindings {
    bindings: Vec<(Identifier, Expression)>,
}

impl LetBindings {
    pub fn new(bindings: Vec<(Identifier, Expression)>) -> Box<LetBindings> {
        Box::new(LetBindings { bindings: bindings })
    }
}

impl Display for LetBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .bindings
            .iter()
            .map(|(ident, expr)| format!("{} = {}", ident, expr))
            .fold(String::from(""), |acc, fmt| {
                if acc == "" {
                    fmt
                } else {
                    format!("{},\n{}", acc, fmt)
                }
            });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Literal {
    Int64Lit(i64),
    StringLit(Box<String>),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int64Lit(num) => f.write_fmt(format_args!("{}", num)),
            Literal::StringLit(string) => f.write_fmt(format_args!("{}", string)),
        }
    }
}
