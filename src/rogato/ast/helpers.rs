#![allow(dead_code)]
#![allow(clippy::vec_box)]

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
use std::rc::Rc;
use std::string::String;

pub fn program(nodes: Vec<Rc<AST>>) -> Program {
    Program::new(nodes)
}

pub fn lit(lit: Literal) -> Rc<Expression> {
    Rc::new(Lit(lit))
}

pub fn int_lit(val: i64) -> Rc<Expression> {
    lit(Int64(val))
}

pub fn string_lit(val: &str) -> Rc<Expression> {
    lit(String(val.to_string()))
}

pub fn tuple_lit(vals: Vec<Rc<Expression>>) -> Rc<Expression> {
    lit(Tuple(TupleItems::from(vals)))
}

pub fn list_lit(vals: Vec<Rc<Expression>>) -> Rc<Expression> {
    lit(List(TupleItems::from(vals)))
}

pub fn struct_lit<S: ToString>(id: S, raw_props: Vec<(S, Rc<Expression>)>) -> Rc<Expression> {
    let mut props = Vec::new();
    for (id, expr) in raw_props {
        props.push((id.to_string(), expr))
    }
    lit(Struct(id.to_string(), Rc::new(StructProps::from(props))))
}

pub fn var(id: &str) -> Rc<Expression> {
    Rc::new(Var(id.to_string()))
}

pub fn const_or_type_ref(id: &str) -> Rc<Expression> {
    Rc::new(ConstOrTypeRef(id.to_string()))
}

pub fn prop_fn_ref(id: &str) -> Rc<Expression> {
    Rc::new(PropFnRef(id.to_string()))
}

pub fn fn_def(id: &str, args: Vec<&str>, body: Rc<Expression>) -> Rc<AST> {
    Rc::new(AST::FnDef(FnDef::new(
        id.to_string(),
        fn_def_args(args),
        body,
    )))
}

pub fn fn_def_args(args: Vec<&str>) -> FnDefArgs {
    FnDefArgs::new(Vec::from_iter(args.iter().map(|a| a.to_string())))
}

pub fn let_expr(bindings: Vec<(&str, Rc<Expression>)>, body: Rc<Expression>) -> Rc<Expression> {
    let bindings: Vec<(String, Rc<Expression>)> = bindings
        .iter()
        .cloned()
        .map(|(name, expr)| (name.to_string(), expr))
        .collect();

    Rc::new(Let(LetExpression::new(LetBindings::new(bindings), body)))
}

pub fn module_def(id: &str, exports: Vec<&str>) -> Rc<AST> {
    Rc::new(AST::ModuleDef(ModuleDef::new(
        id.to_string(),
        module_def_exports(exports),
    )))
}

pub fn module_def_exports(exports: Vec<&str>) -> ModuleExports {
    ModuleExports::new(Vec::from_iter(exports.iter().map(|e| e.to_string())))
}

pub fn call_args(args: Vec<Rc<Expression>>) -> FnCallArgs {
    FnCallArgs::new(args)
}

pub fn fn_call(id: &str, args: Vec<Rc<Expression>>) -> Rc<Expression> {
    Rc::new(Expression::FnCall(id.to_string(), call_args(args)))
}

pub fn op_call(id: &str, left: Rc<Expression>, right: Rc<Expression>) -> Rc<Expression> {
    Rc::new(Expression::OpCall(id.to_string(), left, right))
}

pub fn root_comment(comment: &str) -> Rc<AST> {
    Rc::new(AST::RootComment(comment.to_string()))
}

pub fn commented(comment: &str, exp: Rc<Expression>) -> Rc<Expression> {
    Rc::new(Expression::Commented(comment.to_string(), exp))
}

pub fn type_def(id: &str, type_expr: Rc<TypeExpression>) -> Rc<AST> {
    Rc::new(AST::TypeDef(TypeDef::new(id.to_string(), type_expr)))
}

pub fn tuple_type(items: Vec<Rc<TypeExpression>>) -> Rc<TypeExpression> {
    Rc::new(TypeExpression::TupleType(TupleItems::from(items)))
}

pub fn struct_type(props: Vec<(&str, Rc<TypeExpression>)>) -> Rc<TypeExpression> {
    let boxed_props = Vec::from_iter(
        props
            .iter()
            .map(|(id, expr)| (id.to_string(), expr.clone())),
    );
    Rc::new(TypeExpression::StructType(boxed_props))
}

pub fn int_type() -> Rc<TypeExpression> {
    Rc::new(TypeExpression::IntType)
}

pub fn string_type() -> Rc<TypeExpression> {
    Rc::new(TypeExpression::StringType)
}

pub fn type_ref(id: &str) -> Rc<TypeExpression> {
    Rc::new(TypeExpression::TypeRef(id.to_string()))
}

pub fn query(
    bindings: Vec<(Vec<&str>, Rc<Expression>, bool)>,
    guards: Vec<Rc<Expression>>,
    production: Rc<Expression>,
) -> Rc<Expression> {
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
    Rc::new(Expression::Query(Query::new(
        QueryBindings::new(query_bindings),
        QueryGuards::new(guards),
        production,
    )))
}

pub fn edge_prop(expr: Rc<Expression>, edge: &str) -> Rc<Expression> {
    Rc::new(Expression::EdgeProp(expr, edge.to_string()))
}

pub fn lambda(args: Vec<&str>, body: Rc<Expression>) -> Rc<Expression> {
    let args = args.iter().map(|a| a.to_string()).collect();
    Rc::new(Expression::Lambda(Lambda::new(LambdaArgs::new(args), body)))
}

pub fn symbol(id: &str) -> Rc<Expression> {
    Rc::new(Expression::Symbol(id.to_string()))
}

pub fn quoted(expr: Rc<Expression>) -> Rc<Expression> {
    Rc::new(Expression::Quoted(expr))
}

pub fn quoted_ast(ast: Rc<AST>) -> Rc<Expression> {
    Rc::new(Expression::QuotedAST(ast))
}

pub fn unquoted(expr: Rc<Expression>) -> Rc<Expression> {
    Rc::new(Expression::Unquoted(expr))
}

pub fn unquoted_ast(ast: Rc<AST>) -> Rc<Expression> {
    Rc::new(Expression::UnquotedAST(ast))
}
