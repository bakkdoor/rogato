extern crate peg;

use super::ParserContext;
use peg::{error::ParseError, parser, str::LineCol};
use rogato_common::ast::{
    expression::{
        Expression, FnCallArgs, FnDefArgs, Lambda, LambdaArgs, LetBindings, LetExpression, Literal,
        Query, QueryBinding, QueryBindings, QueryGuards, StructProps, TupleItems,
    },
    fn_def::{FnDef, FnDefBody},
    if_else::IfElse,
    module_def::{ModuleDef, ModuleExports},
    type_expression::{StructTypeProperties, TypeDef, TypeExpression},
    Identifier, Program, AST,
};
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
        = "use " id:struct_identifier() {
            AST::Use(id)
        }

    rule fn_def() -> AST
        = _ "let " _ id:identifier() _ args:(fn_def_arg())* _ "=" _ body:(expression()) _ {
            AST::FnDef(FnDef::new(id, FnDefArgs::new(args), Rc::new(FnDefBody::rogato(Rc::new(body)))))
        }

    rule fn_def_arg() -> Identifier
        = _ id:identifier() _ {
            id
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
        = a:fn_pipe_arg() calls:(fn_pipe_call())+ {
            let call = calls.iter().fold(a, |acc, call|{
                if let Expression::FnCall(id, args) = call {
                    let mut args = args.clone();
                    args.prepend_arg(Rc::new(acc));
                    return Expression::FnCall(id.clone(), args)
                }
                panic!("Failed to create fn call pipeline")
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
        / _ "|>" _ id:identifier() {
            Expression::FnCall(id, FnCallArgs::new(vec![]))
        }

    rule commented_expr() -> Expression
        = c:comment() _ e:expression() {
            Expression::Commented(c, Rc::new(e))
        }

    rule atom() -> Expression
        = if_else()
        / literal_expr()
        / edge_prop()
        / variable()
        / constant_or_type_ref()
        / quoted_expr()
        / "(" _ l:lambda() _ ")" { l }
        / "(" _ c:(fn_pipe() / fn_call() / op_call()) _ ")" { c }


    rule variable() -> Expression
        = id:variable_identifier() {
            Expression::Var(id)
        }
        / "." id:variable_identifier() {
            Expression::PropFnRef(id)
        }

    rule quoted_expr() -> Expression
        = "^" "(" expr:expression() ")" {
            Expression::Quoted(Rc::new(expr))
        }
        / "^" "(" ast:root_def() ")" {
            Expression::QuotedAST(Rc::new(ast))
        }
        / symbol()
        / unquoted_expr()

    rule unquoted_expr() -> Expression
        = "~" "(" expr:expression() ")" {
            Expression::Unquoted(Rc::new(expr))
        }
        / "~" "(" ast:root_def() ")" {
            Expression::UnquotedAST(Rc::new(ast))
        }
        / "~" var:variable() {
            Expression::Unquoted(Rc::new(var))
        }

    rule symbol() -> Expression
        = "^" id:symbol_identifier() {
            Expression::Symbol(id)
        }

    rule query() -> Expression
        = bindings:query_binding()+ guards:query_guard()* _ prod:query_production() {
            Expression::Query(
                Query::new(
                    QueryBindings::new(bindings),
                    QueryGuards::new(guards),
                    Rc::new(prod)
                )
            )
        }

    rule query_binding() -> QueryBinding
        = _ "?" _ vars:query_binding_vars() _ "<!-" _ expr:query_expr() _ {
            QueryBinding::new_negated(vars, Rc::new(expr))
        }
        / _ "?" _ vars:query_binding_vars() _ "<-" _ expr:query_expr() _ {
            QueryBinding::new(vars, Rc::new(expr))
        }

    rule query_binding_vars() -> Vec<Identifier>
        = var:variable_identifier() more_vars:(additional_query_binding_vars())* {
            let mut vars = more_vars;
            vars.insert(0, var);
            vars
        }

    rule additional_query_binding_vars() -> Identifier
        = _ "," _ var:variable_identifier() {
            var
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
        = expr:edge_prop_expr() "#" edge:struct_identifier() {
            Expression::EdgeProp(Rc::new(expr), edge)
        }

    rule edge_prop_expr() -> Expression
        = variable()
        / "(" _ q:query() _ ")" { q }
        / "(" _ c:(fn_pipe() / fn_call() / op_call()) _ ")" { c }

    rule query_guard() -> Rc<Expression>
        = _ c:comment() _ g:query_guard() {
            Rc::new(Expression::Commented(c, g))
        }
        / _ "! " _ expr:query_expr() {
            Rc::new(expr)
        }

    rule query_production() -> Expression
        = c:comment() _ qp:query_production() {
            Expression::Commented(c, Rc::new(qp))
        }
        / "!> " _ expr:query_expr() _ {
            expr
        }


    rule fn_call() -> Expression
        = _ id:identifier() args:(fn_arg())+ _ {
            let args = FnCallArgs::from_owned(args);
            Expression::FnCall(id, args)
        }

    #[cache_left_rec]
    rule op_call() -> Expression
        = left:op_call() " "+ id:operator() ws() right:op_arg() {
            Expression::OpCall(id, Rc::new(left), Rc::new(right))
        }
        / left:op_arg() " "+ id:operator() ws() right:op_arg() {
            Expression::OpCall(id, Rc::new(left), Rc::new(right))
        }
        / left:op_arg() ws() id:operator() " "+ right:op_arg() {
            Expression::OpCall(id, Rc::new(left), Rc::new(right))
        }


    rule fn_arg() -> Expression
        = " "+ e:atom()  { e }

    rule op_arg() -> Expression
        = "(" _ expr:atom() _ ")" {
            expr
        }
        / atom()

    rule let_expr() -> Expression
        = "let" _ bindings:let_bindings() _ "in" _ body:let_body() {
            Expression::Let(
                LetExpression::new(bindings, Rc::new(body))
            )
        }

    rule let_bindings() -> LetBindings
        = binding:let_binding() more_bindings:(additional_let_binding())* {
            let mut bindings = more_bindings;
            bindings.insert(0, binding);
            LetBindings::from_owned(bindings)
        }

    rule additional_let_binding() -> (Identifier, Expression)
        = let_binding_sep()* binding:let_binding() {
            binding
        }

    rule let_binding_sep()
        = " "* "\n"+
        / ","

    rule let_binding() -> (Identifier, Expression)
        = _ id:identifier() _ "=" _ val:let_body() {
            (id, val)
        }
        / _ id:identifier() args:inline_fn_arg()+ _ "=" _ body:let_body() {
            (id.clone(), Expression::InlineFnDef(FnDef::new_inline(id, FnDefArgs::new(args), Rc::new(FnDefBody::rogato(Rc::new(body))))))
        }

    rule inline_fn_arg() -> Identifier
        = " "+ id:identifier() { id }

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
        = c:comment() _ body:let_body() {
            Expression::Commented(c, Rc::new(body))
        }

    rule if_else() -> Expression
        = "if" " "+ cond:if_else_condition() " "+ "then" _ then_expr:atom() _ "else" _ else_expr:atom() {
            Expression::IfElse(IfElse::new(Rc::new(cond), Rc::new(then_expr), Rc::new(else_expr)))
        }

    rule if_else_condition() -> Expression
        = variable()
        / tuple_item()

    rule literal_expr() -> Expression
        = number_lit()
        / bool_lit()
        / string_lit()
        / struct_lit()
        / tuple_lit()
        / list_lit()

    rule number_lit() -> Expression
        = n:$("-"? ['0'..='9']+ "." ['0'..='9']+) {
            let decimal = Decimal::from_str(n).unwrap();
            Expression::Lit(Literal::Number(decimal))
        }
        / n:$("-"? ['0'..='9']+) {
            let decimal = Decimal::from_str(n).unwrap();
            Expression::Lit(Literal::Number(decimal))
        }

    rule bool_lit() -> Expression
        = "true" {
            Expression::Lit(Literal::Bool(true))
        }
        / "false" {
            Expression::Lit(Literal::Bool(false))
        }

    rule string_lit() -> Expression
        = "\"" s:([^ '"']*) "\"" {
            Expression::Lit(Literal::String(String::from_iter(s)))
        }

    rule tuple_lit() -> Expression
        = "{" _ first:tuple_item() rest:(additional_tuple_item())+ _ ("," _)? "}" {
            Expression::Lit(Literal::Tuple(TupleItems::new(first, rest)))
        }

    rule list_lit() -> Expression
        = "[" _ first:tuple_item() rest:(additional_tuple_item())+ _ ("," _)? "]" {
            Expression::Lit(Literal::List(TupleItems::new(first, rest)))
        }
        / "[" _ item:tuple_item() _ "]" {
            Expression::Lit(Literal::List(TupleItems::new(item, vec![])))
        }
        / "[" _ first:tuple_item() _ "::" _ rest:tuple_item() "]" {
            Expression::Lit(Literal::ListCons(Rc::new(first), Rc::new(rest)))
        }
        / "[" _ "]" {
            Expression::Lit(Literal::List(TupleItems::from(vec![])))
        }
        / "[" _ comment() _ "]" {
            Expression::Lit(Literal::List(TupleItems::from(vec![])))
        }

    rule tuple_item() -> Expression
        = fn_call()
        / fn_pipe()
        / op_call()
        / atom()
        / commented_tuple_item()

    rule commented_tuple_item() -> Expression
        = c:comment() _ item:tuple_item() {
            Expression::Commented(c, Rc::new(item))
        }

    rule additional_tuple_item() -> Expression
        = _ "," _ item:tuple_item() {
            item
        }

    rule struct_lit() -> Expression
        = id:struct_identifier() "{" _ first:struct_prop() rest:(additional_struct_prop())*  _ ("," _)? "}" {
            Expression::Lit(Literal::Struct(id, Rc::new(StructProps::new(first, rest))))
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
        = args:lambda_args() " "+ "->" _ body:let_body() {
            Expression::Lambda(Rc::new(Lambda::new(LambdaArgs::new(args), Rc::new(body))))
        }

    rule lambda_args() -> Vec<Identifier>
        = arg:variable_identifier() rest:(additional_lambda_arg())* {
            let mut args = rest;
            args.insert(0, arg);
            args
        }

    rule additional_lambda_arg() -> Identifier
        = " "+ arg:variable_identifier() {
            arg
        }

    rule constant_or_type_ref() -> Expression
        = id:struct_identifier() {
            Expression::ConstOrTypeRef(id)
        }
        / "@" id:struct_identifier() {
            Expression::DBTypeRef(id)
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


    rule _
        = ([' ' | '\t' | '\n'])*

    rule ws()
        = ([' ' | '\t' | '\n'])+

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
