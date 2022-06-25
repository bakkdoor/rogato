extern crate peg;

use peg::{error::ParseError, parser, str::LineCol};

use crate::fmodl::ast::expression::{Expression, FnCallArgs, FnDefArgs, Identifier, Literal};

parser! {
/// Doc comment
grammar parser() for str {
    /// Top level parser rule
    /// This doc comment has multiple lines to test support for that as well
    pub rule root_def() -> Expression
        = fn_def()

    pub rule expression() -> Expression
        = fn_def_body_expr()
        / "(" _ expr:fn_def_body_expr() _ ")" {
            expr
        }

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
        = _ id:identifier() args:(fn_arg())* _ {
            let args = FnCallArgs::new(args);
            Expression::FnCall(id, Box::new(args))
        }

    rule fn_arg() -> Expression
        = " "+ e:expression()  { e }

    rule literal_exp() -> Expression
        = number_lit()
        / string_lit()

    rule number_lit() -> Expression
        = n:$(['0'..='9']+) {
            Expression::Literal(Literal::Int64Lit(n.parse().unwrap()))
        }

    rule string_lit() -> Expression
        = "\"" s:([^ '"']*) "\"" {
            Expression::Literal(Literal::StringLit(String::from_iter(s)))
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


    rule fn_def() -> Expression
        = "let " _ id:identifier() _ args:(fn_def_arg())* _ "=" _ body:(fn_def_body_expr()) {
            Expression::FnDef(id, FnDefArgs::new(args), Box::new(body))
        }

    rule fn_def_arg() -> Identifier
        = _ id:identifier() _ {
            id
        }

    rule fn_def_body_expr() -> Expression
        = fn_call()
        / sum()
}}

pub fn parse(str: &str) -> Result<Expression, ParseError<LineCol>> {
    parser::root_def(str)
}

pub fn parse_expr(str: &str) -> Result<Expression, ParseError<LineCol>> {
    parser::expression(str)
}
