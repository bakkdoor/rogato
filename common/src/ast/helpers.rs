use rust_decimal::Decimal;

use crate::ast::expression::{
    FnCall, FnCallArgs, Lambda, LambdaArgs, LambdaVariant, LetBindings, LetExpression, Query,
    QueryBinding, QueryBindings, QueryGuards, StructProps, TupleItems,
};
use crate::ast::fn_def::FnDef;
use crate::ast::module_def::{ModuleDef, ModuleExports};
use crate::ast::type_expression::TypeDef;
use crate::ast::{
    expression::{
        Expression::{self, *},
        FnDefArgs,
        Literal::{self, *},
    },
    AST,
};
use crate::ast::{type_expression::TypeExpression, Program};
use std::rc::Rc;

use super::expression::{IfElse, MapKVPair};
use super::fn_def::FnDefBody;
use super::pattern::Pattern;
use super::type_expression::StructTypeProperties;
use super::{Identifier, VarIdentifier};

pub fn program<Nodes: IntoIterator<Item = Rc<AST>>>(nodes: Nodes) -> Program {
    Program::from_iter(nodes)
}

pub fn lit(lit: Literal) -> Rc<Expression> {
    Rc::new(Lit(lit))
}

pub fn number_lit<Num>(val: Num) -> Rc<Expression>
where
    Decimal: From<Num>,
{
    lit(Number(Decimal::from(val)))
}
pub fn bool_lit(val: bool) -> Rc<Expression> {
    lit(Bool(val))
}

pub fn string_lit(val: &str) -> Rc<Expression> {
    lit(String(val.to_string()))
}

pub fn tuple_lit<Iter: IntoIterator<Item = Rc<Expression>>>(vals: Iter) -> Rc<Expression> {
    lit(Tuple(TupleItems::from_iter(vals)))
}

pub fn list_lit<Iter: IntoIterator<Item = Rc<Expression>>>(vals: Iter) -> Rc<Expression> {
    lit(List(TupleItems::from_iter(vals)))
}

pub fn list_cons(first: Rc<Expression>, rest: Rc<Expression>) -> Rc<Expression> {
    lit(ListCons(first, rest))
}

pub fn struct_lit<S: Into<Identifier>, Props: IntoIterator<Item = (S, Rc<Expression>)>>(
    id: S,
    raw_props: Props,
) -> Rc<Expression> {
    let mut props = Vec::new();
    for (id, expr) in raw_props.into_iter() {
        props.push((id.into(), expr))
    }
    lit(Struct(id.into(), Rc::new(StructProps::from(props))))
}

pub fn map_lit<Iter: IntoIterator<Item = (Rc<Expression>, Rc<Expression>)>>(
    items: Iter,
) -> Rc<Expression> {
    let kv_pairs: Vec<Rc<MapKVPair<Expression>>> = items
        .into_iter()
        .map(|kvp| Rc::new(MapKVPair::new(kvp.0, kvp.1)))
        .collect();
    lit(Map(TupleItems::from_iter(kv_pairs)))
}

pub fn map_cons_lit<Iter: IntoIterator<Item = (Rc<Expression>, Rc<Expression>)>>(
    items: Iter,
    rest: Rc<Expression>,
) -> Rc<Expression> {
    let kv_pairs: Vec<Rc<MapKVPair<Expression>>> = items
        .into_iter()
        .map(|kvp| Rc::new(MapKVPair::new(kvp.0, kvp.1)))
        .collect();
    lit(MapCons(TupleItems::from_iter(kv_pairs), rest))
}

pub fn var(id: &str) -> Rc<Expression> {
    Rc::new(Var(id.into()))
}

pub fn const_or_type_ref(id: &str) -> Rc<Expression> {
    Rc::new(ConstOrTypeRef(id.into()))
}

pub fn db_type_ref(id: &str) -> Rc<Expression> {
    Rc::new(DBTypeRef(id.into()))
}

pub fn prop_fn_ref(id: &str) -> Rc<Expression> {
    Rc::new(PropFnRef(id.into()))
}

pub fn fn_def<P: Into<Rc<Pattern>>, Args: IntoIterator<Item = P>>(
    id: &str,
    args: Args,
    body: Rc<Expression>,
) -> Rc<AST> {
    Rc::new(AST::FnDef(FnDef::new(
        id,
        fn_def_args(args),
        Rc::new(FnDefBody::rogato(body)),
    )))
}

pub fn fn_def_args<P: Into<Rc<Pattern>>, Args: IntoIterator<Item = P>>(args: Args) -> FnDefArgs {
    FnDefArgs::new(Vec::from_iter(args.into_iter().map(|a| a.into())))
}

pub fn if_else(
    cond: Rc<Expression>,
    then_expr: Rc<Expression>,
    else_expr: Rc<Expression>,
) -> Rc<Expression> {
    Rc::new(Expression::IfElse(IfElse::new(cond, then_expr, else_expr)))
}
pub fn let_expr<
    VarName: Into<VarIdentifier>,
    Bindings: IntoIterator<Item = (VarName, Rc<Expression>)>,
>(
    bindings: Bindings,
    body: Rc<Expression>,
) -> Rc<Expression> {
    let bindings: Vec<(VarIdentifier, Rc<Expression>)> = bindings
        .into_iter()
        .map(|(name, expr)| (name.into(), expr))
        .collect();

    Rc::new(Let(LetExpression::new(LetBindings::new(bindings), body)))
}

pub fn module_def<Exports: IntoIterator<Item = &'static str>>(
    id: &str,
    exports: Exports,
) -> Rc<AST> {
    Rc::new(AST::ModuleDef(ModuleDef::new(
        id.into(),
        module_def_exports(exports),
    )))
}

pub fn module_def_exports<Exports: IntoIterator<Item = &'static str>>(
    exports: Exports,
) -> ModuleExports {
    ModuleExports::new(Vec::from_iter(exports.into_iter().map(|e| e.into())))
}

pub fn call_args<Args: IntoIterator<Item = Rc<Expression>>>(args: Args) -> FnCallArgs {
    FnCallArgs::new(args)
}

pub fn fn_call<Args: IntoIterator<Item = Rc<Expression>>>(id: &str, args: Args) -> Rc<Expression> {
    Rc::new(Expression::FnCall(FnCall::new(id.into(), call_args(args))))
}

pub fn op_call(id: &str, left: Rc<Expression>, right: Rc<Expression>) -> Rc<Expression> {
    Rc::new(Expression::OpCall(id.into(), left, right))
}

pub fn root_comment(comment: &str) -> Rc<AST> {
    Rc::new(AST::RootComment(comment.to_string()))
}

pub fn commented(comment: &str, exp: Rc<Expression>) -> Rc<Expression> {
    Rc::new(Expression::Commented(comment.to_string(), exp))
}

pub fn type_def(id: &str, type_expr: Rc<TypeExpression>) -> Rc<AST> {
    Rc::new(AST::TypeDef(TypeDef::new(id.into(), type_expr)))
}

pub fn db_type_def(id: &str, type_expr: Rc<TypeExpression>) -> Rc<AST> {
    Rc::new(AST::TypeDef(TypeDef::new(id.into(), type_expr)))
}

pub fn tuple_type<Items: IntoIterator<Item = Rc<TypeExpression>>>(
    items: Items,
) -> Rc<TypeExpression> {
    Rc::new(TypeExpression::TupleType(TupleItems::from_iter(items)))
}

pub fn list_type(type_expr: Rc<TypeExpression>) -> Rc<TypeExpression> {
    Rc::new(TypeExpression::ListType(type_expr))
}

pub fn struct_type<Iter: IntoIterator<Item = (&'static str, Rc<TypeExpression>)>>(
    props: Iter,
) -> Rc<TypeExpression> {
    Rc::new(TypeExpression::StructType(StructTypeProperties::new(
        props.into_iter().map(|(id, expr)| (id.into(), expr)),
    )))
}

pub fn int_type() -> Rc<TypeExpression> {
    Rc::new(TypeExpression::NumberType)
}

pub fn string_type() -> Rc<TypeExpression> {
    Rc::new(TypeExpression::StringType)
}

pub fn type_ref(id: &str) -> Rc<TypeExpression> {
    Rc::new(TypeExpression::TypeRef(id.into()))
}

pub fn query<
    BindIds: Into<Vec<&'static str>>,
    Binds: IntoIterator<Item = (BindIds, Rc<Expression>, bool)>,
    Guards: IntoIterator<Item = Rc<Expression>>,
>(
    bindings: Binds,
    guards: Guards,
    production: Rc<Expression>,
) -> Rc<Expression> {
    let query_bindings = bindings
        .into_iter()
        .map(|(ids, expr, is_negated)| {
            let qb_ids = ids.into().iter().map(|id| id.into()).collect();
            if is_negated {
                QueryBinding::new_negated(qb_ids, expr)
            } else {
                QueryBinding::new(qb_ids, expr)
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
    Rc::new(Expression::EdgeProp(expr, edge.into()))
}

pub fn lambda<Args: IntoIterator<Item = &'static str>>(
    args: Args,
    body: Rc<Expression>,
) -> Rc<Expression> {
    let args = args.into_iter().map(|a| Rc::new(a.into())).collect();
    Rc::new(Expression::Lambda(Rc::new(Lambda::new(vec![Rc::new(
        LambdaVariant::new(LambdaArgs::new(args), body),
    )]))))
}

pub fn lambda_<Args: IntoIterator<Item = Rc<Pattern>>>(
    args: Args,
    body: Rc<Expression>,
) -> Rc<LambdaVariant> {
    let args = args.into_iter().collect();
    Rc::new(LambdaVariant::new(LambdaArgs::new(args), body))
}

pub fn lambda_p<
    Args: IntoIterator<Item = Rc<Pattern>>,
    Variants: IntoIterator<Item = (Args, Rc<Expression>)>,
>(
    variants: Variants,
) -> Rc<Expression> {
    let variants = variants
        .into_iter()
        .map(|(args, body)| {
            let args: LambdaArgs<Rc<Pattern>> = LambdaArgs::new(args.into_iter().collect());
            Rc::new(LambdaVariant::new(args, Rc::clone(&body)))
        })
        .collect();
    Rc::new(Expression::Lambda(Rc::new(Lambda::new(variants))))
}

pub fn symbol(id: &str) -> Rc<Expression> {
    Rc::new(Expression::Symbol(id.into()))
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

pub fn inline_fn_def<Args: IntoIterator<Item = Rc<Pattern>>>(
    id: &str,
    args: Args,
    body: Rc<Expression>,
) -> Rc<Expression> {
    Rc::new(Expression::InlineFnDef(FnDef::new_inline(
        id,
        fn_def_args(args),
        Rc::new(FnDefBody::rogato(body)),
    )))
}

pub fn any_p() -> Rc<Pattern> {
    Rc::new(Pattern::Any)
}

pub fn list_cons_p(head_pattern: Rc<Pattern>, tail_pattern: Rc<Pattern>) -> Rc<Pattern> {
    Rc::new(Pattern::ListCons(head_pattern, tail_pattern))
}

pub fn empty_list_p() -> Rc<Pattern> {
    Rc::new(Pattern::EmptyList)
}

pub fn list_lit_p<P: Into<Vec<Rc<Pattern>>>>(patterns: P) -> Rc<Pattern> {
    Rc::new(Pattern::List(TupleItems::from(patterns.into())))
}

pub fn tuple_lit_p<P: Into<Vec<Rc<Pattern>>>>(patterns: P) -> Rc<Pattern> {
    let patterns = patterns.into();
    Rc::new(Pattern::Tuple(patterns.len(), TupleItems::from(patterns)))
}

pub fn map_lit_p<P: Into<Vec<(Rc<Pattern>, Rc<Pattern>)>>>(kv_pairs: P) -> Rc<Pattern> {
    let patterns = kv_pairs
        .into()
        .iter()
        .map(|(key, val)| Rc::new(MapKVPair::new(Rc::clone(key), Rc::clone(val))))
        .collect();
    Rc::new(Pattern::Map(TupleItems::from(patterns)))
}

pub fn map_cons_lit_p<P: Into<Vec<(Rc<Pattern>, Rc<Pattern>)>>>(
    kv_pairs: P,
    tail_pattern: Rc<Pattern>,
) -> Rc<Pattern> {
    let patterns = kv_pairs
        .into()
        .iter()
        .map(|(key, val)| Rc::new(MapKVPair::new(Rc::clone(key), Rc::clone(val))))
        .collect();
    Rc::new(Pattern::MapCons(TupleItems::from(patterns), tail_pattern))
}

pub fn var_p(id: &str) -> Rc<Pattern> {
    Rc::new(Pattern::Var(id.into()))
}

pub fn number_p<N: Into<Decimal>>(n: N) -> Rc<Pattern> {
    Rc::new(Pattern::Number(n.into()))
}

pub fn bool_p(b: bool) -> Rc<Pattern> {
    Rc::new(Pattern::Bool(b))
}

pub fn string_p<S: ToString>(s: S) -> Rc<Pattern> {
    Rc::new(Pattern::String(s.to_string()))
}

pub fn symbol_p<S: Into<Identifier>>(s: S) -> Rc<Pattern> {
    Rc::new(Pattern::Symbol(s.into()))
}

pub fn p<P: Into<Vec<Pattern>>>(vec: P) -> Vec<Rc<Pattern>> {
    let vec = vec.into();
    let mut patterns = Vec::with_capacity(vec.len());
    for pat in vec {
        patterns.push(Rc::new(pat))
    }
    patterns
}

pub fn vars(ids: &[&str]) -> Vec<Rc<Pattern>> {
    let mut vec = Vec::with_capacity(ids.len());
    for id in ids.iter() {
        let id: VarIdentifier = id.into();
        vec.push(Rc::new(Pattern::Var(id)))
    }
    vec
}
