#[cfg(test)]
pub mod db;
#[cfg(test)]
pub mod parser;

use crate::rogato::ast::expression::{
    FnCallArgs, Lambda, LambdaArgs, LetBindings, LetExpression, Query, QueryBinding, QueryBindings,
    QueryGuards, StructProps, TupleItems,
};
use crate::rogato::ast::fn_def::FnDef;
use crate::rogato::ast::module_def::{ModuleDef, ModuleExports};
use crate::rogato::ast::type_expression::TypeDef;
use crate::rogato::ast::{
    expression::{
        Expression::{self, *},
        FnDefArgs,
        Literal::{self, *},
    },
    AST,
};
use crate::rogato::ast::{type_expression::TypeExpression, Program};
pub use crate::rogato::parser::{parse, parse_expr};
use std::string::String;

#[macro_export]
macro_rules! assert_parse {
    ($code:expr, $expected:expr) => {
        assert_eq!(
            crate::rogato::parser::parse($code),
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
            crate::rogato::parser::parse_ast($code),
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
            crate::rogato::parser::parse_expr($code),
            Ok($expected),
            "Expected expression code to parse: {:?}",
            $code
        )
    };
}

pub fn program(nodes: Vec<Box<AST>>) -> Program {
    Program::from_boxed(nodes)
}

pub fn lit(lit: Literal) -> Box<Expression> {
    Box::new(Lit(lit))
}

pub fn int_lit(val: i64) -> Box<Expression> {
    lit(Int64(val))
}

pub fn string_lit(val: &str) -> Box<Expression> {
    lit(String(val.to_string()))
}

pub fn tuple_lit(vals: Vec<Box<Expression>>) -> Box<Expression> {
    lit(Tuple(TupleItems::from(vals)))
}

pub fn list_lit(vals: Vec<Box<Expression>>) -> Box<Expression> {
    lit(List(TupleItems::from(vals)))
}

pub fn struct_lit<S: ToString>(id: S, raw_props: Vec<(S, Box<Expression>)>) -> Box<Expression> {
    let mut props = Vec::new();
    for (id, expr) in raw_props {
        props.push((id.to_string(), expr))
    }
    lit(Struct(id.to_string(), Box::new(StructProps::from(props))))
}

pub fn var(id: &str) -> Box<Expression> {
    Box::new(Var(id.to_string()))
}

pub fn const_or_type_ref(id: &str) -> Box<Expression> {
    Box::new(ConstOrTypeRef(id.to_string()))
}

pub fn prop_fn_ref(id: &str) -> Box<Expression> {
    Box::new(PropFnRef(id.to_string()))
}

pub fn sum(a: Box<Expression>, b: Box<Expression>) -> Box<Expression> {
    Box::new(Sum(a, b))
}

pub fn product(a: Box<Expression>, b: Box<Expression>) -> Box<Expression> {
    Box::new(Product(a, b))
}

pub fn fn_def(id: &str, args: Vec<&str>, body: Box<Expression>) -> Box<AST> {
    Box::new(AST::FnDef(FnDef::new(
        id.to_string(),
        fn_def_args(args),
        body,
    )))
}

pub fn fn_def_args(args: Vec<&str>) -> FnDefArgs {
    FnDefArgs::new(Vec::from_iter(args.iter().map(|a| a.to_string())))
}

pub fn let_expr(bindings: Vec<(&str, Box<Expression>)>, body: Box<Expression>) -> Box<Expression> {
    let bindings: Vec<(String, Expression)> = bindings
        .iter()
        .cloned()
        .map(|(name, expr)| (name.to_string(), *expr))
        .collect();

    Box::new(Let(LetExpression::new(LetBindings::new(bindings), body)))
}

pub fn module_def(id: &str, exports: Vec<&str>) -> Box<AST> {
    Box::new(AST::ModuleDef(ModuleDef::new(
        id.to_string(),
        module_def_exports(exports),
    )))
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
    Box::new(AST::TypeDef(TypeDef::new(id.to_string(), type_expr)))
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

pub fn query(
    bindings: Vec<(Vec<&str>, Box<Expression>, bool)>,
    guards: Vec<Box<Expression>>,
    production: Box<Expression>,
) -> Box<Expression> {
    let guards: Vec<Expression> = Vec::from_iter(guards.iter().map(|g| *g.clone()));
    let query_bindings = bindings
        .iter()
        .map(|(ids, expr, is_negated)| {
            let qb_ids = ids.iter().map(|id| id.to_string()).collect();
            if *is_negated {
                QueryBinding::new_negated(qb_ids, expr.clone())
            } else {
                QueryBinding::new(qb_ids, expr.clone())
            }
        })
        .collect();
    Box::new(Expression::Query(Query::new(
        QueryBindings::new(query_bindings),
        QueryGuards::new(guards),
        production,
    )))
}

pub fn edge_prop(expr: Box<Expression>, edge: &str) -> Box<Expression> {
    Box::new(Expression::EdgeProp(expr, edge.to_string()))
}

pub fn lambda(args: Vec<&str>, body: Box<Expression>) -> Box<Expression> {
    let args = args.iter().map(|a| a.to_string()).collect();
    Box::new(Expression::Lambda(Lambda::new(LambdaArgs::new(args), body)))
}

pub fn symbol(id: &str) -> Box<Expression> {
    Box::new(Expression::Symbol(id.to_string()))
}

pub fn quoted(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Quoted(expr))
}

pub fn quoted_ast(ast: Box<AST>) -> Box<Expression> {
    Box::new(Expression::QuotedAST(ast))
}

pub fn unquoted(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Unquoted(expr))
}

pub fn unquoted_ast(ast: Box<AST>) -> Box<Expression> {
    Box::new(Expression::UnquotedAST(ast))
}
