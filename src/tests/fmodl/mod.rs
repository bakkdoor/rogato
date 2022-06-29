#[cfg(test)]
pub mod parser;

use crate::fmodl::ast::expression::{FnCallArgs, LetBindings, StructProps, TupleItems};
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
use crate::fmodl::ast::{type_expression::TypeExpression, Program};
pub use crate::fmodl::parser::{parse, parse_expr};

#[macro_export]
macro_rules! assert_parse {
    ($code:expr, $expected:expr) => {
        assert_eq!(
            crate::fmodl::parser::parse($code),
            Ok($expected),
            "Expected program code to parse: {:?}",
            $code
        )
    };
}

#[macro_export]
macro_rules! assert_parse_ast {
    ($code:expr, $expected:expr) => {
        assert_eq!(
            crate::fmodl::parser::parse_ast($code),
            Ok($expected),
            "Expected code to parse: {:?}",
            $code
        )
    };
}

#[macro_export]
macro_rules! assert_parse_expr {
    ($code:expr, $expected:expr) => {
        assert_eq!(
            crate::fmodl::parser::parse_expr($code),
            Ok($expected),
            "Expected expression code to parse: {:?}",
            $code
        )
    };
}

pub fn program(nodes: Vec<Box<AST>>) -> Program {
    Program::from(nodes)
}

pub fn lit(lit: Literal) -> Box<Expression> {
    Box::new(Lit(lit))
}

pub fn int_lit(val: i64) -> Box<Expression> {
    lit(Int64Lit(val))
}

pub fn string_lit(val: &str) -> Box<Expression> {
    lit(StringLit(Box::new(val.to_string())))
}

pub fn tuple_lit(vals: Vec<Box<Expression>>) -> Box<Expression> {
    lit(TupleLit(TupleItems::from(vals)))
}

pub fn struct_lit<S: ToString>(id: S, raw_props: Vec<(S, Box<Expression>)>) -> Box<Expression> {
    let mut props = Vec::new();
    for (id, expr) in raw_props {
        props.push((id.to_string(), expr))
    }
    lit(StructLit(
        id.to_string(),
        Box::new(StructProps::from(props)),
    ))
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

pub fn fn_def(id: &str, args: Vec<&str>, body: Box<Expression>) -> Box<AST> {
    Box::new(FnDef(id.to_string(), fn_def_args(args), body))
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

pub fn module_def(id: &str, exports: Vec<&str>) -> Box<AST> {
    Box::new(ModuleDef(id.to_string(), module_def_exports(exports)))
}

pub fn module_def_exports(exports: Vec<&str>) -> ModuleExports {
    ModuleExports::new(Vec::from_iter(exports.iter().map(|e| e.to_string())))
}

pub fn call_args(args: Vec<Box<Expression>>) -> Box<FnCallArgs> {
    let mut args_unboxed = Vec::new();
    for a in args {
        args_unboxed.push(*a)
    }
    Box::new(FnCallArgs::new(args_unboxed))
}

pub fn fn_call(id: &str, args: Vec<Box<Expression>>) -> Box<Expression> {
    Box::new(Expression::FnCall(id.to_string(), call_args(args)))
}

pub fn op_call(id: &str, left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::OpCall(id.to_string(), left, right))
}

pub fn root_comment(comment: &str) -> Box<AST> {
    Box::new(AST::RootComment(comment.to_string()))
}

pub fn commented(comment: &str, exp: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Commented(comment.to_string(), exp))
}

pub fn type_def(id: &str, type_expr: Box<TypeExpression>) -> Box<AST> {
    Box::new(AST::TypeDef(id.to_string(), type_expr))
}

pub fn tuple_type(items: Vec<Box<TypeExpression>>) -> Box<TypeExpression> {
    Box::new(TypeExpression::TupleType(TupleItems::from(items)))
}

pub fn struct_type(props: Vec<(&str, Box<TypeExpression>)>) -> Box<TypeExpression> {
    let boxed_props = Vec::from_iter(
        props
            .iter()
            .map(|(id, expr)| (id.to_string(), expr.clone())),
    );
    Box::new(TypeExpression::StructType(boxed_props))
}

pub fn int_type() -> Box<TypeExpression> {
    Box::new(TypeExpression::IntType)
}

pub fn string_type() -> Box<TypeExpression> {
    Box::new(TypeExpression::StringType)
}

pub fn type_ref(id: &str) -> Box<TypeExpression> {
    Box::new(TypeExpression::TypeRef(id.to_string()))
}
