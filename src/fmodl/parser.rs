extern crate peg;

use peg::{error::ParseError, parser, str::LineCol};

use crate::fmodl::ast::expression::{Expression, FnCallArgs, FnDefArgs, LetBindings, Literal};
use crate::fmodl::ast::module_def::ModuleExports;
use crate::fmodl::ast::{Identifier, AST};

parser! {
/// Doc comment
grammar parser() for str {
    /// Top level parser rule
    /// This doc comment has multiple lines to test support for that as well
    pub rule root_def() -> AST
        = module_def()
        / fn_def()

    rule module_def() -> AST
        = "module " _ id:identifier() _ exports:module_exports() _ {
            AST::ModuleDef(id, ModuleExports::new(exports))
        }
        / "module " _ id:identifier() _ "(" _ ")" _ {
            AST::ModuleDef(id, ModuleExports::new(vec![]))
        }
        / "module " _ id:identifier() _ {
            AST::ModuleDef(id, ModuleExports::new(vec![]))
        }
    rule module_exports() -> Vec<Identifier>
        = "(" _ first_export:identifier() more_exports:(additional_module_export())* _ ")" {
            let mut exports = Vec::new();
            exports.push(first_export);
            exports.append(&mut more_exports.to_owned());
            exports
        }

    rule additional_module_export() -> Identifier
        = _ "," _ id:identifier() {
            id
        }

    rule fn_def() -> AST
        = "let " _ id:identifier() _ args:(fn_def_arg())* _ "=" _ body:(expression()) {
            AST::FnDef(id, FnDefArgs::new(args), Box::new(body))
        }

    rule fn_def_arg() -> Identifier
        = _ id:identifier() _ {
            id
        }

    pub rule expression() -> Expression
        = let_exp()
        / fn_call()
        / sum()
        / variable()
        / literal_exp()

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
        = variable()
        / number_lit()
        / "(" _ v:sum() _ ")" { v }
        / "(" _ c:fn_call() _ ")" { c }

    rule variable() -> Expression
        = _ id:identifier() _ {
            Expression::Var(id)
        }

    rule fn_call() -> Expression
        = _ id:identifier() args:(fn_arg())+ _ {
            let args = FnCallArgs::new(args);
            Expression::FnCall(id, Box::new(args))
        }

    rule fn_arg() -> Expression
        = " "+ e:expression()  { e }

    rule let_exp() -> Expression
        = "let " _ bindings:let_bindings() _ "in" _ body:expression() {
            Expression::Let(LetBindings::new(bindings), Box::new(body))
        }

    rule let_bindings() -> Vec<(Identifier, Expression)>
        = binding:let_binding() more_bindings:(additional_let_binding())* {
            let mut bindings = Vec::new();
            bindings.push(binding);
            bindings.append(&mut more_bindings.to_owned());
            bindings
        }

    rule additional_let_binding() -> (Identifier, Expression)
        = _ "," _ binding:let_binding() {
            binding
        }

    rule let_binding() -> (Identifier, Expression)
        = _ id:identifier() _ "=" _ val:expression() _ {
            (id, val)
        }

    rule literal_exp() -> Expression
        = number_lit()
        / string_lit()

    rule number_lit() -> Expression
        = n:$(['0'..='9']+) {
            Expression::Lit(Literal::Int64Lit(n.parse().unwrap()))
        }

    rule string_lit() -> Expression
        = "\"" s:([^ '"']*) "\"" {
            Expression::Lit(Literal::StringLit(String::from_iter(s)))
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

pub fn parse(str: &str) -> Result<AST, ParseError<LineCol>> {
    parser::root_def(str)
}

pub fn parse_expr(str: &str) -> Result<Box<Expression>, ParseError<LineCol>> {
    match parser::expression(str) {
        Ok(expr) => Ok(Box::new(expr)),
        Err(err) => Err(err),
    }
}
