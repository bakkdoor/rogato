extern crate peg;

use crate::rogato::ast::{
    expression::{
        Expression, FnCallArgs, FnDefArgs, LambdaArgs, LetBindings, Literal, QueryBinding,
        QueryBindings, QueryGuards, StructProps, TupleItems,
    },
    module_def::ModuleExports,
    type_expression::TypeExpression,
    Identifier, Program, AST,
};
use peg::{error::ParseError, parser, str::LineCol};

parser! {
/// Doc comment
grammar parser() for str {
    pub rule program() -> Program
        = _ defs:(program_root_def())* _ {
            Program::from(defs)
        }
        / _ {
            Program::new(vec![])
        }

    pub rule program_root_def() -> Box<AST>
        = _ def:root_def() {
            Box::new(def)
        }

    pub rule root_def() -> AST
        = module_def()
        / fn_def()
        / type_def()
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
            prepend_vec(first_export, &mut more_exports.to_owned())
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

    rule type_def() -> AST
        = _ "type " _ id:identifier() _ "::" _ t_expr:type_expr() {
            AST::TypeDef(id, Box::new(t_expr))
        }

    rule type_expr() -> TypeExpression
        = "String" { TypeExpression::StringType }
        / "Int" { TypeExpression::IntType }
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
            TypeExpression::ListType(Box::new(type_expr))
        }

    rule function_type() -> TypeExpression
        = "(" _ arg_types:(tuple_type_item())+ " "+ "->" return_type:type_expr() _ ")"{
            TypeExpression::FunctionType(LambdaArgs::new(arg_types), Box::new(return_type))
        }

    rule struct_type() -> TypeExpression
        = "{" _ properties:(struct_prop_type())+ "}" {
            TypeExpression::StructType(properties)
        }

    rule struct_prop_type() -> (Identifier, Box<TypeExpression>)
        = id:identifier() " "+ "::" _ type_expr:type_expr() " "* "," _ {
            (id, Box::new(type_expr))
        }
        / id:identifier() " "+ "::" _ type_expr:type_expr() [^'\n']* "\n"+ _ {
            (id, Box::new(type_expr))
        }

    pub rule expression() -> Expression
        = let_expr()
        / query()
        / fn_call()
        / op_call()
        / lambda()
        / sum()
        / variable()
        / literal_expr()
        / commented_expr()

    rule commented_expr() -> Expression
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
        / literal_expr()
        / constant_or_type_ref()
        / "(" _ v:sum() _ ")" { v }
        / "(" _ c:(fn_call() / op_call()) _ ")" { c }
        / "(" _ l:lambda() _ ")" { l }

    rule variable() -> Expression
        = id:variable_identifier() {
            Expression::Var(id)
        }

    rule query() -> Expression
        = bindings:query_binding()+ guards:query_guard()* _ prod:query_production() {
            Expression::Query(
                Box::new(QueryBindings::new(Box::new(bindings))),
                Box::new(QueryGuards::new(guards)),
                Box::new(prod)
            )
        }

    rule query_binding() -> QueryBinding
        = _ "?" _ vars:query_binding_vars() _ "<-" _ expr:query_expr() {
            QueryBinding::new(vars, Box::new(expr))
        }

    rule query_binding_vars() -> Vec<Identifier>
        = var:variable_identifier() more_vars:(additional_query_binding_vars())* {
            prepend_vec(var, &mut more_vars.to_owned())
        }

    rule additional_query_binding_vars() -> Identifier
        = _ "," _ var:variable_identifier() {
            var
        }

    rule query_expr() -> Expression
        = edge_prop()
        / atom()

    rule edge_prop() -> Expression
        = expr:edge_prop_expr() "#" edge:struct_identifier() {
            Expression::EdgeProp(Box::new(expr), edge)
        }

    rule edge_prop_expr() -> Expression
        = variable()

    rule query_guard() -> Expression
        = _ c:comment() _ g:query_guard() {
            Expression::Commented(c, Box::new(g))
        }
        / _ "! " _ expr:query_expr() {
            expr
        }

    rule query_production() -> Expression
        = c:comment() _ qp:query_production() {
            Expression::Commented(c, Box::new(qp))
        }
        / "!> " _ expr:query_expr() {
            expr
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
        / literal_expr()
        / variable()

    rule let_expr() -> Expression
        = "let" _ bindings:let_bindings() _ "in" _ body:let_body() {
            Expression::Let(LetBindings::new(bindings), Box::new(body))
        }

    rule let_bindings() -> Vec<(Identifier, Expression)>
        = binding:let_binding() more_bindings:(additional_let_binding())* {
            prepend_vec(binding, &mut more_bindings.to_owned())
        }

    rule let_body() -> Expression
        = fn_call()
        / op_call()
        / lambda()
        / sum()
        / variable()
        / literal_expr()
        / commented_let_body()
        / query()

    rule commented_let_body() -> Expression
        = c:comment() _ body:let_body() {
            Expression::Commented(c, Box::new(body))
        }

    rule additional_let_binding() -> (Identifier, Expression)
        = _ "," _ binding:let_binding() {
            binding
        }

    rule let_binding() -> (Identifier, Expression)
        = _ id:identifier() _ "=" _ val:let_body() _ {
            (id, val)
        }

    rule literal_expr() -> Expression
        = number_lit()
        / string_lit()
        / struct_lit()
        / tuple_lit()
        / list_lit()

    rule number_lit() -> Expression
        = n:$(['0'..='9']+) {
            Expression::Lit(Literal::Int64Lit(n.parse().unwrap()))
        }

    rule string_lit() -> Expression
        = "\"" s:([^ '"']*) "\"" {
            Expression::Lit(Literal::StringLit(Box::new(String::from_iter(s))))
        }

    rule tuple_lit() -> Expression
        = "{" _ first:tuple_item() rest:(additional_tuple_item())+ _ ("," _)? "}" {
            Expression::Lit(Literal::TupleLit(TupleItems::new(first, rest)))
        }

    rule list_lit() -> Expression
        = "[" _ first:tuple_item() rest:(additional_tuple_item())+ _ ("," _)? "]" {
            Expression::Lit(Literal::ListLit(TupleItems::new(first, rest)))
        }
        / "[" _ item:tuple_item() _ "]" {
            Expression::Lit(Literal::ListLit(TupleItems::new(item, vec![])))
        }
        / "[" _ "]" {
            Expression::Lit(Literal::ListLit(TupleItems::from(vec![])))
        }

    rule tuple_item() -> Expression
        = fn_call()
        / op_call()
        / sum()
        / atom()
        / commented_tuple_item()

    rule commented_tuple_item() -> Expression
        = c:comment() _ item:tuple_item() {
            Expression::Commented(c, Box::new(item))
        }

    rule additional_tuple_item() -> Expression
        = _ "," _ item:tuple_item() {
            item
        }

    rule struct_lit() -> Expression
        = id:struct_identifier() "{" _ first:struct_prop() rest:(additional_struct_prop())*  _ ("," _)? "}" {
            Expression::Lit(Literal::StructLit(id, Box::new(StructProps::new(first, rest))))
        }

    rule additional_struct_prop() -> (Identifier, Expression)
        = _ "," _ prop:struct_prop() {
            prop
        }

    rule struct_prop() -> (Identifier, Expression)
        = id:identifier() _ ":" _ expr:(tuple_item()) {
            (id, expr)
        }

    rule lambda() -> Expression
        = id:identifier() " "+ "->" _ body:let_body() {
            Expression::Lambda(LambdaArgs::new(vec![id]), Box::new(body))
        }

    rule constant_or_type_ref() -> Expression
        = id:struct_identifier() {
            Expression::ConstOrTypeRef(id)
        }

    rule struct_identifier() -> Identifier
        = id1:$([ 'A'..='Z' ]) id2:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9' | '.' | '@' | '$'])* {
            join_string(id1, id2)
        }

    rule variable_identifier() -> Identifier
        = id1:$([ 'a'..='z' ]) id2:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9' | '.' | '@' | '$'])* {
            join_string(id1, id2)
        }

    rule identifier() -> Identifier
        = id1:$([ 'a'..='z' | 'A'..='Z' | '-' | '_' | '@' | '$']) id2:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9' | '.' | '@' | '$'])* {
            join_string(id1, id2)
        }

    rule operator() -> Identifier
        = id:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '!' | '^' | '='])+ {
            String::from_iter(id)
        }

    rule _
        = ([' ' | '\t' | '\n'])*

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
pub type ParseASTResult = Result<Box<AST>, ParseError<LineCol>>;

#[cfg(test)]
pub fn parse_ast(str: &str) -> ParseASTResult {
    parser::program_root_def(str)
}

pub type ParseExprResult = Result<Box<Expression>, ParseError<LineCol>>;

pub fn parse_expr(str: &str) -> ParseExprResult {
    match parser::expression(str) {
        Ok(expr) => Ok(Box::new(expr)),
        Err(err) => Err(err),
    }
}

fn join_string(first: &str, rest: Vec<&str>) -> String {
    let mut s = String::new();
    s.push_str(first);
    s.push_str(String::from_iter(rest).as_str());
    s
}

fn prepend_vec<T>(first: T, rest: &mut Vec<T>) -> Vec<T> {
    let mut joined = Vec::new();
    joined.push(first);
    joined.append(rest);
    joined
}
