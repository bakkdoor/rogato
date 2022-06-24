extern crate peg;

use peg::{error::ParseError, parser, str::LineCol};

use crate::fmodl::ast::expression::{Expression, FunctionArgs, Identifier, Literal};

parser! {
/// Doc comment
grammar parser() for str {
    /// Top level parser rule
    /// This doc comment has multiple lines to test support for that as well
    pub rule expression() -> Expression
        = sum()
        / fn_call()

    rule _ = [' ' | '\n']*

    rule sum() -> Expression
        = l:product() _ "+" _ r:product() {
            Expression::Sum(Box::new(l), Box::new(r))
        }
        / product()

    rule product() -> Expression
        = l:atom() _ "*" _ r:atom() {
            Expression::Product(Box::new(l), Box::new(r))
        }
        / atom()

    rule atom() -> Expression
        = number_lit()
        / "(" _ v:sum() _ ")" { v }
        / variable()
        / fn_call()

    rule variable() -> Expression
        = _ id:identifier() _ {
            Expression::Var(id)
        }

    rule fn_call() -> Expression
        = "(" _ id:(operator() / identifier()) _ args:(fn_arg())* _ ")" {
            let args = FunctionArgs::new(args);
            Expression::FnCall(id, Box::new(args))
        }

    rule fn_arg() -> Expression
        = _ e:expression() { e }

    rule literal_exp() -> Expression
        = number_lit()
        / string_lit()

    rule number_lit() -> Expression
        = n:$(['0'..='9']+) {
            Expression::Literal(Literal::Int64(n.parse().unwrap()))
        }

    rule string_lit() -> Expression
        = "\"" s:([^ '"']*) "\"" {
            Expression::Literal(Literal::String(String::from_iter(s)))
        }

    rule identifier() -> Identifier
        = id1:$([ 'a'..='z' | 'A'..='Z' | '-' | '_']) id2:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9'])* {
            let mut id = String::new();
            id.push_str(id1);
            id.push_str(String::from_iter(id2).as_str());
            id
        }

    rule operator() -> Identifier
        = id:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '!' | '^'])+ {
            String::from_iter(id)
        }


}}

pub fn parse(str: &str) -> Result<Expression, ParseError<LineCol>> {
    parser::expression(str)
}
