extern crate peg;

use peg::{error::ParseError, parser, str::LineCol};

use crate::fmodl::ast::expression::{Expression, FnCallArgs, FnDefArgs, LetBindings, Literal};
use crate::fmodl::ast::module_def::ModuleExports;
use crate::fmodl::ast::{Identifier, Program, AST};

parser! {
/// Doc comment
grammar parser() for str {
    pub rule program() -> Program
        = _ defs:(additional_root_def())* _ {
            let nodes = Vec::from_iter(defs.iter().map(|d| Box::new(d.clone())));
            Program::new(nodes)
        }
        / _ {
            Program::new(vec![])
        }

    rule additional_root_def() -> AST
        = _ def:root_def() {
            def
        }

    pub rule root_def() -> AST
        = module_def()
        / fn_def()
        / c:comment() {
            AST::RootComment(c)
        }

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
        = _ "let " _ id:identifier() _ args:(fn_def_arg())* _ "=" _ body:(expression()) _ {
            AST::FnDef(id, FnDefArgs::new(args), Box::new(body))
        }

    rule fn_def_arg() -> Identifier
        = _ id:identifier() _ {
            id
        }

    pub rule expression() -> Expression
        = let_exp()
        / fn_call()
        / op_call()
        / sum()
        / variable()
        / literal_exp()
        / commented_exp()

    rule commented_exp() -> Expression
        = c:comment() _ e:expression() {
            Expression::Commented(c, Box::new(e))
        }

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
        / literal_exp()
        / "(" _ v:sum() _ ")" { v }
        / "(" _ c:(fn_call() / op_call()) _ ")" { c }

    rule variable() -> Expression
        = id:identifier() {
            Expression::Var(id)
        }

    rule fn_call() -> Expression
        = _ id:identifier() args:(fn_arg())+ _ {
            let args = FnCallArgs::new(args);
            Expression::FnCall(id, Box::new(args))
        }

    rule op_call() -> Expression
        = left:op_arg() " "+ id:operator() _ right:op_arg() {
            Expression::OpCall(id, Box::new(left), Box::new(right))
        }
        / left:op_arg() _ id:operator() " "+ right:op_arg() {
            Expression::OpCall(id, Box::new(left), Box::new(right))
        }

    rule fn_arg() -> Expression
        = " "+ e:atom()  { e }

    rule op_arg() -> Expression
        = "(" _ expr:atom() _ ")" {
            expr
        }
        / sum()
        / literal_exp()
        / variable()

    rule let_exp() -> Expression
        = "let " _ bindings:let_bindings() _ "in" _ body:let_body() {
            Expression::Let(LetBindings::new(bindings), Box::new(body))
        }

    rule let_bindings() -> Vec<(Identifier, Expression)>
        = binding:let_binding() more_bindings:(additional_let_binding())* {
            let mut bindings = Vec::new();
            bindings.push(binding);
            bindings.append(&mut more_bindings.to_owned());
            bindings
        }

    rule let_body() -> Expression
        = fn_call()
        / op_call()
        / sum()
        / variable()
        / literal_exp()
        / commented_let_body()

    rule commented_let_body() -> Expression
        = c:comment() body:let_body() {
            Expression::Commented(c, Box::new(body))
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
            Expression::Lit(Literal::StringLit(Box::new(String::from_iter(s))))
        }

    rule identifier() -> Identifier
        = id1:$([ 'a'..='z' | 'A'..='Z' | '-' | '_']) id2:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9'])* {
            let mut id = String::new();
            id.push_str(id1);
            id.push_str(String::from_iter(id2).as_str());
            id
        }

    rule operator() -> Identifier
        = id:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '!' | '^' | '='])+ {
            String::from_iter(id)
        }

    rule _
        = ([' ' | '\n'])*

    rule comment() -> String
        = [' ' | '\t']* "//" comment:([^ '\n'])* {
            String::from_iter(comment)
        }

}}

pub type ParseResult = Result<Program, ParseError<LineCol>>;

pub fn parse(str: &str) -> ParseResult {
    parser::program(str)
}

#[cfg(test)]
pub type ParseASTResult = Result<AST, ParseError<LineCol>>;

#[cfg(test)]
pub fn parse_ast(str: &str) -> ParseASTResult {
    parser::root_def(str)
}

pub type ParseExprResult = Result<Box<Expression>, ParseError<LineCol>>;

pub fn parse_expr(str: &str) -> ParseExprResult {
    match parser::expression(str) {
        Ok(expr) => Ok(Box::new(expr)),
        Err(err) => Err(err),
    }
}
