#[cfg(test)]
pub mod parser;

use crate::fmodl::ast::expression::{FnCallArgs, LetBindings};
use crate::fmodl::ast::module_def::ModuleExports;
use crate::fmodl::ast::AST::{FnDef, ModuleDef};
use crate::fmodl::ast::{
    expression::{
        Expression::{self, *},
        FnDefArgs,
        Literal::{self, *},
    },
    AST,
};
use crate::fmodl::parser::{parse, parse_expr};

pub fn assert_parse(code: &str, expected: AST) {
    assert_eq!(
        parse(code),
        Ok(expected),
        "Expected code to parse: {:?}",
        code
    )
}

pub fn assert_parse_expr(code: &str, expected: Box<Expression>) {
    assert_eq!(
        parse_expr(code),
        Ok(expected),
        "Expected expression code to parse: {:?}",
        code
    )
}

pub fn lit(lit: Literal) -> Box<Expression> {
    Box::new(Lit(lit))
}

pub fn var(id: &str) -> Box<Expression> {
    Box::new(Var(id.to_string()))
}

pub fn sum(a: Box<Expression>, b: Box<Expression>) -> Box<Expression> {
    Box::new(Sum(a, b))
}

pub fn product(a: Box<Expression>, b: Box<Expression>) -> Box<Expression> {
    Box::new(Product(a, b))
}

pub fn fn_def(id: &str, args: Vec<&str>, body: Box<Expression>) -> AST {
    FnDef(id.to_string(), fn_def_args(args), body)
}

pub fn fn_def_args(args: Vec<&str>) -> FnDefArgs {
    FnDefArgs::new(Vec::from_iter(args.iter().map(|a| a.to_string())))
}

pub fn let_exp(bindings: Vec<(&str, Box<Expression>)>, body: Box<Expression>) -> Box<Expression> {
    let bindings: Vec<(String, Expression)> = bindings
        .iter()
        .cloned()
        .map(|(name, expr)| (name.to_string(), *expr))
        .collect();

    Box::new(Let(LetBindings::new(bindings), body))
}

pub fn module_def(id: &str, exports: Vec<&str>) -> AST {
    ModuleDef(id.to_string(), module_def_exports(exports))
}

pub fn module_def_exports(exports: Vec<&str>) -> ModuleExports {
    ModuleExports::new(Vec::from_iter(exports.iter().map(|e| e.to_string())))
}

pub fn call_args(args: Vec<Box<Expression>>) -> FnCallArgs {
    let mut args_unboxed = Vec::new();
    for a in args {
        args_unboxed.push(*a)
    }
    FnCallArgs::new(args_unboxed)
}

pub fn fn_call(id: &str, args: Vec<Box<Expression>>) -> Box<Expression> {
    Box::new(Expression::FnCall(
        id.to_string(),
        Box::new(call_args(args)),
    ))
}

pub fn op_call(id: &str, args: Vec<Box<Expression>>) -> Box<Expression> {
    Box::new(Expression::OpCall(
        id.to_string(),
        Box::new(call_args(args)),
    ))
}
