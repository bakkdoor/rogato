extern crate peg;

use peg::{error::ParseError, parser, str::LineCol};

use crate::fmodl::ast::expression::{Expression, LiteralExpr};

parser! {
/// Doc comment
grammar parser() for str {
    /// Top level parser rule
    /// This doc comment has multiple lines to test support for that as well
    pub rule expression() -> Expression
        = sum()

    rule _ = [' ' | '\n']*

    rule sum() -> Expression
        = l:product() _ "+" _ r:product() { Expression::Sum(Box::new(l), Box::new(r)) }
        / product()

    rule product() -> Expression
        = l:atom() _ "*" _ r:atom() { Expression::Product(Box::new(l), Box::new(r)) }
        / atom()

    rule atom() -> Expression
        = number_lit()
        / "(" _ v:sum() _ ")" { v }

    rule literal_exp() -> Expression
        = number_lit()
        / string_lit()

    rule number_lit() -> Expression
        = n:$(['0'..='9']+) { Expression::LiteralExpr(LiteralExpr::Int64(n.parse().unwrap())) }

    rule string_lit() -> Expression
        = "\"" s:([^ '"']*) "\"" { Expression::LiteralExpr(LiteralExpr::String(String::from_iter(s))) }
}}

pub fn parse(str: &str) -> Result<Expression, ParseError<LineCol>> {
    parser::expression(str)
}
