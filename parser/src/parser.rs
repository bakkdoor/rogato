extern crate peg;

use super::ParserContext;
use peg::{error::ParseError, parser, str::LineCol};
use rogato_common::ast::{
    expression::{
        ExprKind, Expression, FnCall, FnCallArgs, FnDefArgs, Lambda, LambdaArgs, LambdaVariant,
        LetBindings, LetExpression, Literal, Query, QueryBinding, QueryBindings, QueryGuards,
        StructProps, TupleItems,
    },
    fn_def::{FnDef, FnDefBody},
    if_else::IfElse,
    literal::MapKVPair,
    module_def::{ModuleDef, ModuleExports},
    pattern::Pattern,
    type_expression::{StructTypeProperties, TypeDef, TypeExpression},
    Identifier, Program, VarIdentifier, AST,
};
use rogato_common::span::Span;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use smol_str::SmolStr;
use std::rc::Rc;

parser! {
/// Doc comment
grammar parser(context: &ParserContext) for str {
    rule traced<T>(e: rule<T>) -> T =
    &(input:$([_]*) {
        #[cfg(feature = "trace")]
        println!("[PEG_INPUT_START]\n{}\n[PEG_TRACE_START]", input);
    })
    e:e()? {?
        #[cfg(feature = "trace")]
        println!("[PEG_TRACE_STOP]");
        e.ok_or("")
    }

    pub rule traced_program() -> Program
        = traced(<program()>)

    pub rule program() -> Program
        = _ defs:(program_root_def())* _ {
            Program::new(defs)
        }
        / _ {
            Program::new(vec![])
        }

    pub rule program_root_def() -> Rc<AST>
        = _ def:root_def() {
            Rc::new(def)
        }

    pub rule root_def() -> AST
        = fn_def()
        / type_def()
        / module_def()
        / use_stmt()
        / c:comment() {
            AST::RootComment(c)
        }

    rule module_def() -> AST
        = "module " _ id:identifier() _ exports:module_exports() _ {
            AST::ModuleDef(ModuleDef::new(id, ModuleExports::new(exports)))
        }
        / "module " _ id:identifier() _ "{" _ "}" _ {
            AST::ModuleDef(ModuleDef::new(id, ModuleExports::new(vec![])))
        }
        / "module " _ id:identifier() _ {
            AST::ModuleDef(ModuleDef::new(id, ModuleExports::new(vec![])))
        }

    rule module_exports() -> Vec<Identifier>
        = "{" _ first_export:identifier() more_exports:(additional_module_export())* _ "}" {
            let mut exports = more_exports;
            exports.insert(0, first_export);
            exports
        }

    rule additional_module_export() -> Identifier
        = _ "," _ id:identifier() {
            id
        }

    rule use_stmt() -> AST
        = "use" s() id:struct_identifier() s() "{" _ imports:(identifier() ** (_ "," _)) _ "}" {
            AST::Use(id, imports)
        }
        / "use" s() id:struct_identifier() {
            AST::Use(id, vec![])
        }

    rule fn_def() -> AST
        = _ "let " _ id:identifier() _ args:(pattern() ** s()) _ "=" _ body:(expression()) _ {
            AST::FnDef(FnDef::new(id, FnDefArgs::new(args), Rc::new(FnDefBody::rogato(Rc::new(body)))))
        }

    rule list_sep()
        = (s()? "," _)
        / (_ "," s()?)

    rule pattern() -> Rc<Pattern>
        = "(" _ p:pattern() _ ")" {
            p
        }
        / n:number_lit() {
            Rc::new(Pattern::Number(n))
        }
        / b:bool_lit() {
            Rc::new(Pattern::Bool(b))
        }
        / s:string_lit() {
            Rc::new(Pattern::String(s))
        }
        / "_" {
            Rc::new(Pattern::Any)
        }
        / "[" _ "]" {
            Rc::new(Pattern::EmptyList)
        }
        / "[" _ head:pattern() _ "::" _ tail:pattern() _ "]" {
            Rc::new(Pattern::ListCons(head,tail))
        }
        / "[" _ items:(pattern() ** list_sep()) _ "]" {
            Rc::new(Pattern::List(TupleItems::from(items)))
        }
        / "{" _ tail:pattern() _ "::" _ items:(kv_pattern() ** list_sep()) _ "}" {
            Rc::new(Pattern::MapCons(TupleItems::from(items), tail))
        }
        / "{" _ items:(kv_pattern() ** list_sep()) _ "}" {
            Rc::new(Pattern::Map(TupleItems::from(items)))
        }
        / "{" _ items:(pattern() ** list_sep()) _ "}" {
            Rc::new(Pattern::Tuple(items.len(), TupleItems::from(items)))
        }
        / id:variable_identifier() {
            if id == "_" {
                Rc::new(Pattern::Any)
            } else {
                Rc::new(Pattern::Var(VarIdentifier::new(id)))
            }
        }
        / "^" id:symbol_identifier() {
            Rc::new(Pattern::Symbol(id))
        }

    rule kv_pattern() -> Rc<MapKVPair<Pattern>>
        = key:pattern() _ ":" _ val:pattern() {
            Rc::new(MapKVPair::new(key, val))
        }

    rule type_def() -> AST
        = _ "type " _ id:type_identifier() _ "::" _ t_expr:type_expr() {
            AST::TypeDef(TypeDef::new(id, Rc::new(t_expr)))
        }

    rule type_expr() -> TypeExpression
        = "Bool" { TypeExpression::BoolType }
        / "Int" { TypeExpression::NumberType }
        / "String" { TypeExpression::StringType }
        / tuple_type()
        / list_type()
        / function_type()
        / struct_type()
        / id:identifier() {
            TypeExpression::TypeRef(id)
        }

    rule tuple_type_item() -> TypeExpression
        = type_expr()

    rule additional_tuple_type_item() -> TypeExpression
        = " "* "," _ item:tuple_type_item() {
            item
        }

    rule tuple_type() -> TypeExpression
        = "{" _ first:tuple_type_item() rest:(additional_tuple_type_item())+ _ ("," _)? "}" {
            TypeExpression::TupleType(TupleItems::new(first, rest))
        }

    rule list_type() -> TypeExpression
        = "[" _ type_expr:type_expr() _ "]" {
            TypeExpression::ListType(Rc::new(type_expr))
        }

    rule function_type() -> TypeExpression
        = "(" _ arg_types:(tuple_type_item())+ " "+ "->" return_type:type_expr() _ ")"{
            TypeExpression::FunctionType(LambdaArgs::new(arg_types), Rc::new(return_type))
        }

    rule struct_type() -> TypeExpression
        = "{" _ properties:(struct_prop_type())+ "}" {
            TypeExpression::StructType(StructTypeProperties::new(properties))
        }

    rule struct_prop_type() -> (Identifier, Rc<TypeExpression>)
        = id:identifier() " "+ "::" _ type_expr:type_expr() " "* "," _ {
            (id, Rc::new(type_expr))
        }
        / id:identifier() " "+ "::" _ type_expr:type_expr() [^'\n']* "\n"+ _ {
            (id, Rc::new(type_expr))
        }

    pub rule expression() -> Expression
        = if_else()
        / fn_pipe()
        / let_expr()
        / query()
        / lambda()
        / fn_call()
        / op_call()
        / atom()
        / commented_expr()

    rule fn_pipe() -> Expression
        = start:position!() a:fn_pipe_arg() calls:(fn_pipe_call())+ end:position!() {
            let span = Span::new(start, end);
            let call = calls.iter().fold(a, |acc, call|{
                if let ExprKind::FnCall(fn_call) = &call.kind {
                    let mut args = fn_call.args.clone();
                    args.prepend_arg(Rc::new(acc));
                    Expression::new(ExprKind::FnCall(FnCall::new(fn_call.id.clone(), args)), span)
                } else {
                    panic!("Failed to create fn call pipeline")
                }
            });

            call
        }

    rule fn_pipe_arg() -> Expression
        = lambda()
        / fn_call()
        / atom()

    rule fn_pipe_call() -> Expression
        = _ "|>" _ fc:fn_call() {
            fc
        }
        / _ "|>" _ start:position!() id:identifier() end:position!() {
            Expression::new(ExprKind::FnCall(FnCall::new(id, FnCallArgs::empty())), Span::new(start, end))
        }

    rule commented_expr() -> Expression
        = start:position!() c:comment() _ e:expression() end:position!() {
            Expression::new(ExprKind::Commented(c, Rc::new(e)), Span::new(start, end))
        }

    rule atom() -> Expression
        = if_else()
        / literal_expr()
        / edge_prop()
        / variable()
        / constant_or_type_ref()
        / quoted_expr()
        / lambda()
        / "(" _ c:(fn_pipe() / fn_call() / op_call()) _ ")" { c }


    rule variable() -> Expression
        = start:position!() id:variable_identifier() end:position!() {
            Expression::new(ExprKind::Var(id.into()), Span::new(start, end))
        }
        / start:position!() "." id:variable_identifier() end:position!() {
            Expression::new(ExprKind::PropFnRef(id), Span::new(start, end))
        }

    rule quoted_expr() -> Expression
        = start:position!() "^" "(" expr:expression() ")" end:position!() {
            Expression::new(ExprKind::Quoted(Rc::new(expr)), Span::new(start, end))
        }
        / start:position!() "^" "(" ast:root_def() ")" end:position!() {
            Expression::new(ExprKind::QuotedAST(Rc::new(ast)), Span::new(start, end))
        }
        / symbol()
        / unquoted_expr()

    rule unquoted_expr() -> Expression
        = start:position!() "~" "(" expr:expression() ")" end:position!() {
            Expression::new(ExprKind::Unquoted(Rc::new(expr)), Span::new(start, end))
        }
        / start:position!() "~" "(" ast:root_def() ")" end:position!() {
            Expression::new(ExprKind::UnquotedAST(Rc::new(ast)), Span::new(start, end))
        }
        / start:position!() "~" var:variable() end:position!() {
            Expression::new(ExprKind::Unquoted(Rc::new(var)), Span::new(start, end))
        }

    rule symbol() -> Expression
        = start:position!() "^" id:symbol_identifier() end:position!() {
            Expression::new(ExprKind::Symbol(id), Span::new(start, end))
        }

    rule query() -> Expression
        = start:position!() bindings:query_binding()+ guards:query_guard()* _ prod:query_production() end:position!() {
            Expression::new(ExprKind::Query(
                Query::new(
                    QueryBindings::new(bindings),
                    QueryGuards::new(guards),
                    Rc::new(prod)
                )
            ), Span::new(start, end))
        }

    rule query_binding() -> QueryBinding
        = _ "?" _ vars:query_binding_vars() _ "<!-" _ expr:query_expr() _ {
            QueryBinding::new_negated(vars, Rc::new(expr))
        }
        / _ "?" _ vars:query_binding_vars() _ "<-" _ expr:query_expr() _ {
            QueryBinding::new(vars, Rc::new(expr))
        }

    rule query_binding_vars() -> Vec<VarIdentifier>
        = var:variable_identifier() more_vars:(additional_query_binding_vars())* {
            let mut vars = more_vars;
            vars.insert(0, VarIdentifier::new(var));
            vars
        }

    rule additional_query_binding_vars() -> VarIdentifier
        = _ "," _ var:variable_identifier() {
            VarIdentifier::new(var)
        }

    rule query_expr() -> Expression
        = edge_prop()
        / "(" _ l:lambda() _ ")" { l }
        / "(" _ q:query() _ ")" { q }
        / "(" _ c:(fn_pipe() / fn_call() / op_call()) _ ")" { c }
        / fn_pipe()
        / fn_call()
        / constant_or_type_ref()
        / lambda()
        / variable()
        / quoted_expr()
        / op_call()
        / literal_expr()

    rule edge_prop() -> Expression
        = start:position!() expr:edge_prop_expr() "#" edge:struct_identifier() end:position!() {
            Expression::new(ExprKind::EdgeProp(Rc::new(expr), edge), Span::new(start, end))
        }

    rule edge_prop_expr() -> Expression
        = variable()
        / "(" _ q:query() _ ")" { q }
        / "(" _ c:(fn_pipe() / fn_call() / op_call()) _ ")" { c }

    rule query_guard() -> Rc<Expression>
        = start:position!() _ c:comment() _ g:query_guard() end:position!() {
            Expression::rc_spanned(ExprKind::Commented(c, g), Span::new(start, end))
        }
        / _ "! " _ expr:query_expr() {
            Rc::new(expr)
        }

    rule query_production() -> Expression
        = start:position!() c:comment() _ qp:query_production() end:position!() {
            Expression::new(ExprKind::Commented(c, Rc::new(qp)), Span::new(start, end))
        }
        / "!> " _ expr:query_expr() _ {
            expr
        }


    rule fn_call() -> Expression
        = start:position!() _ ids:(identifier() ** ".") args:(fn_arg())+ _ end:position!() {
            let args = FnCallArgs::from_owned(args);
            Expression::new(ExprKind::FnCall(FnCall::new(ids.join(".").into(), args)), Span::new(start, end))
        }
        / start:position!() _ id:identifier() args:(fn_arg())+ _ end:position!() {
            let args = FnCallArgs::from_owned(args);
            Expression::new(ExprKind::FnCall(FnCall::new(id, args)), Span::new(start, end))
        }

    #[cache_left_rec]
    rule op_call() -> Expression
        = start:position!() left:op_call() " "+ id:operator() ws() right:op_arg() end:position!() {
            Expression::new(ExprKind::OpCall(id, Rc::new(left), Rc::new(right)), Span::new(start, end))
        }
        / start:position!() left:op_arg() " "+ id:operator() ws() right:op_arg() end:position!() {
            Expression::new(ExprKind::OpCall(id, Rc::new(left), Rc::new(right)), Span::new(start, end))
        }
        / start:position!() left:op_arg() ws() id:operator() " "+ right:op_arg() end:position!() {
            Expression::new(ExprKind::OpCall(id, Rc::new(left), Rc::new(right)), Span::new(start, end))
        }


    rule fn_arg() -> Expression
        = " "+ e:atom()  { e }

    rule op_arg() -> Expression
        = "(" _ expr:atom() _ ")" {
            expr
        }
        / atom()

    rule let_expr() -> Expression
        = start:position!() "let" _ bindings:let_bindings() _ "in" _ body:let_body() end:position!() {
            Expression::new(ExprKind::Let(
                LetExpression::new(bindings, Rc::new(body))
            ), Span::new(start, end))
        }

    rule let_bindings() -> LetBindings
        = binding:let_binding() more_bindings:(additional_let_binding())* {
            let mut bindings = more_bindings;
            bindings.insert(0, binding);
            LetBindings::from_owned(bindings)
        }

    rule additional_let_binding() -> (VarIdentifier, Expression)
        = let_binding_sep()* binding:let_binding() {
            binding
        }

    rule let_binding_sep()
        = " "* "\n"+
        / ","

    rule let_binding() -> (VarIdentifier, Expression)
        = _ id:identifier() _ "=" _ val:let_body() {
            (VarIdentifier::new(id.clone()), val)
        }
        / start:position!() _ id:identifier() _ args:(pattern() ** s()) _ "=" _ body:let_body() end:position!() {
            (VarIdentifier::new(id.clone()), Expression::new(ExprKind::InlineFnDef(FnDef::new_inline(id, FnDefArgs::new(args), Rc::new(FnDefBody::rogato(Rc::new(body))))), Span::new(start, end)))
        }

    rule let_body() -> Expression
        = lambda()
        / if_else()
        / query()
        / fn_pipe()
        / fn_call()
        / op_call()
        / atom()
        / commented_let_body()

    rule commented_let_body() -> Expression
        = start:position!() c:comment() _ body:let_body() end:position!() {
            Expression::new(ExprKind::Commented(c, Rc::new(body)), Span::new(start, end))
        }

    rule if_else() -> Expression
        = start:position!() "if" " "+ cond:if_else_condition() " "+ "then" _ then_expr:atom() _ "else" _ else_expr:atom() end:position!() {
            Expression::new(ExprKind::IfElse(IfElse::new(Rc::new(cond), Rc::new(then_expr), Rc::new(else_expr))), Span::new(start, end))
        }

    rule if_else_condition() -> Expression
        = variable()
        / tuple_item()

    rule literal_expr() -> Expression
        = number_lit_expr()
        / map_lit_expr()
        / bool_lit_expr()
        / string_lit_expr()
        / struct_lit_expr()
        / tuple_lit_expr()
        / list_lit_expr()

    rule number_lit_expr() -> Expression
        = start:position!() n:number_lit() end:position!() {
            Expression::new(ExprKind::Lit(Literal::Number(n)), Span::new(start, end))
        }

    rule bool_lit_expr() -> Expression
        = start:position!() b:bool_lit() end:position!() {
            Expression::new(ExprKind::Lit(Literal::Bool(b)), Span::new(start, end))
        }

    rule string_lit_expr() -> Expression
        = start:position!() s:string_lit() end:position!() {
            Expression::new(ExprKind::Lit(Literal::String(s)), Span::new(start, end))
        }

    rule tuple_lit_expr() -> Expression
        = start:position!() "{" _ first:tuple_item() rest:(additional_tuple_item())+ _ ("," _)? "}" end:position!() {
            Expression::new(ExprKind::Lit(Literal::Tuple(TupleItems::new(first, rest))), Span::new(start, end))
        }

    rule list_lit_expr() -> Expression
        = start:position!() "[" _ first:tuple_item() rest:(additional_tuple_item())+ _ ("," _)? "]" end:position!() {
            Expression::new(ExprKind::Lit(Literal::List(TupleItems::new(first, rest))), Span::new(start, end))
        }
        / start:position!() "[" _ item:tuple_item() _ "]" end:position!() {
            Expression::new(ExprKind::Lit(Literal::List(TupleItems::new(item, vec![]))), Span::new(start, end))
        }
        / start:position!() "[" _ first:tuple_item() _ "::" _ rest:tuple_item() "]" end:position!() {
            Expression::new(ExprKind::Lit(Literal::ListCons(Rc::new(first), Rc::new(rest))), Span::new(start, end))
        }
        / start:position!() "[" _ "]" end:position!() {
            Expression::new(ExprKind::Lit(Literal::List(TupleItems::from(vec![]))), Span::new(start, end))
        }
        / start:position!() "[" _ comment() _ "]" end:position!() {
            Expression::new(ExprKind::Lit(Literal::List(TupleItems::from(vec![]))), Span::new(start, end))
        }

    rule map_lit_expr() -> Expression
        = start:position!() "{" _ kv_pairs:(kv_pair() ** (_ "," _)) _ "}" end:position!() {
            Expression::new(ExprKind::Lit(Literal::Map(TupleItems::from(kv_pairs))), Span::new(start, end))
        }
        / start:position!() "{" _ rest:tuple_item() _ "::" _ kv_pairs:(kv_pair() ** (_ "," _)) _ "}" end:position!() {
            Expression::new(ExprKind::Lit(Literal::MapCons(TupleItems::from(kv_pairs), Rc::new(rest))), Span::new(start, end))
        }

    rule kv_pair() -> Rc<MapKVPair<Expression>>
        = key:tuple_item() _ ":" _ value:tuple_item() {
            Rc::new(MapKVPair::new(Rc::new(key), Rc::new(value)))
        }

    rule tuple_item() -> Expression
        = fn_call()
        / fn_pipe()
        / op_call()
        / atom()
        / commented_tuple_item()

    rule commented_tuple_item() -> Expression
        = start:position!() c:comment() _ item:tuple_item() end:position!() {
            Expression::new(ExprKind::Commented(c, Rc::new(item)), Span::new(start, end))
        }

    rule additional_tuple_item() -> Expression
        = _ "," _ item:tuple_item() {
            item
        }

    rule struct_lit_expr() -> Expression
        = start:position!() id:struct_identifier() "{" _ first:struct_prop() rest:(additional_struct_prop())*  _ ("," _)? "}" end:position!() {
            Expression::new(ExprKind::Lit(Literal::Struct(id, Rc::new(StructProps::new(first, rest)))), Span::new(start, end))
        }

    rule additional_struct_prop() -> (Identifier, Rc<Expression>)
        = _ "," _ prop:struct_prop() {
            prop
        }

    rule struct_prop() -> (Identifier, Rc<Expression>)
        = id:identifier() _ ":" _ expr:(tuple_item()) {
            (id, Rc::new(expr))
        }

    rule lambda() -> Expression
        = start:position!() "(" _ variants:(lambda_variant() ** (_ "," _)) _ ")" end:position!() {
            Expression::new(ExprKind::Lambda(Rc::new(Lambda::new(variants))), Span::new(start, end))
        }
        / start:position!() variant:lambda_variant() end:position!() {
            Expression::new(ExprKind::Lambda(Rc::new(Lambda::new(vec![variant]))), Span::new(start, end))
        }

    rule lambda_variant() -> Rc<LambdaVariant>
        = args:lambda_args() s() "->" _ body:let_body() {
            Rc::new(LambdaVariant::new(LambdaArgs::new(args), Rc::new(body)))
        }
        / "->" _ body:let_body() {
            Rc::new(LambdaVariant::new(LambdaArgs::empty(), Rc::new(body)))
        }
        / "(" _ v:lambda_variant() _ ")" {
            v
        }

    rule lambda_args() -> Vec<Rc<Pattern>>
        = arg:lambda_arg() rest:(additional_lambda_arg())* {
            let mut args = rest;
            args.insert(0, arg);
            args
        }

    rule additional_lambda_arg() -> Rc<Pattern>
        = " "+ arg:lambda_arg() {
            arg
        }

    rule lambda_arg() -> Rc<Pattern>
        = p:pattern() {
            p
        }

    rule constant_or_type_ref() -> Expression
        = start:position!() id:struct_identifier() end:position!() {
            if is_qualified_fn_call(&id) {
                Expression::new(ExprKind::FnCall(FnCall::new(id, FnCallArgs::empty())), Span::new(start, end))
            } else {
                Expression::new(ExprKind::ConstOrTypeRef(id), Span::new(start, end))
            }
        }
        / start:position!() "@" id:struct_identifier() end:position!() {
            Expression::new(ExprKind::DBTypeRef(id), Span::new(start, end))
        }

    rule struct_identifier() -> Identifier
        = id1:$([ 'A'..='Z' ]) id2:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9' | '.' | '@' | '$'])* {
            join_string(id1, id2)
        }

    rule type_identifier() -> Identifier
        = struct_identifier()
        / "@" id:struct_identifier() {
            join_string("@", vec![id.as_str()])
        }

    rule variable_identifier() -> Identifier
        = id1:$([ 'a'..='z' ]) id2:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9' | '.' | '@' | '$'])* {
            join_string(id1, id2)
        }
        / id:"_" {
            SmolStr::from("_")
        }

    rule symbol_identifier() -> Identifier
        = variable_identifier()
        / struct_identifier()
        / identifier()

    rule identifier() -> Identifier
        = id1:$([ 'a'..='z' | 'A'..='Z' | '-' | '_' | '@' | '$']) id2:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9' | '.' | '@' | '$'])* {
            join_string(id1, id2)
        }

    rule operator() -> Identifier
        = "!" id:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '!' | '^' | '=' | '|' | '%'])+ {
            join_string("!", id)
        }
        / id:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '^' | '=' | '|' | '%'])+ id2:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '!' | '^' | '=' | '|' | '%'])* {
            join_string(SmolStr::from_iter(id).as_str(), id2)
        }

    rule number_lit() -> Decimal
        = n:$("-"? ['0'..='9']+ "." ['0'..='9']+) {
            Decimal::from_str(n).unwrap()
        }
        / n:$("-"? ['0'..='9']+) {
            Decimal::from_str(n).unwrap()
        }

    rule bool_lit() -> bool
        = "true" {
            true
        }
        / "false" {
            false
        }

    rule string_lit() -> String
        = "\"" s:([^ '"']*) "\"" {
            String::from_iter(s)
        }

    rule _
        = ([' ' | '\t' | '\n'])*

    rule ws()
        = ([' ' | '\t' | '\n'])+

    rule s()
        = ([' ' | '\t'])+


    rule comment() -> String
        = [' ' | '\t']* "//" comment:([^ '\n'])* {
            String::from_iter(comment)
        }
}}

pub type ParseResult = Result<Program, ParseError<LineCol>>;

#[cfg(not(feature = "trace"))]
pub fn parse(str: &str, context: &ParserContext) -> ParseResult {
    parser::program(str, context)
}

#[cfg(feature = "trace")]
pub fn parse(str: &str, context: &ParserContext) -> ParseResult {
    parser::traced_program(str, context)
}

pub type ParseASTResult = Result<Rc<AST>, ParseError<LineCol>>;

pub fn parse_ast(str: &str, context: &ParserContext) -> ParseASTResult {
    parser::program_root_def(str, context)
}

pub type ParseExprResult = Result<Rc<Expression>, ParseError<LineCol>>;

pub fn parse_expr(str: &str, context: &ParserContext) -> ParseExprResult {
    match parser::expression(str, context) {
        Ok(expr) => Ok(Rc::new(expr)),
        Err(err) => Err(err),
    }
}

fn join_string(first: &str, rest: Vec<&str>) -> SmolStr {
    let mut parts = vec![first];
    for part in rest {
        parts.push(part)
    }
    SmolStr::from_iter(parts)
}

fn is_qualified_fn_call(id: &Identifier) -> bool {
    let id_parts: Vec<&str> = id.split('.').collect();
    if let Some(last) = id_parts.last() {
        let mut ci = last.char_indices();
        if let Some((0, c)) = ci.next() {
            if c.is_lowercase() {
                return true;
            }
        }
    }

    false
}
