extern crate peg;

use crate::rogato::ast::{
    expression::{
        Expression, FnCallArgs, FnDefArgs, Lambda, LambdaArgs, LetBindings, LetExpression, Literal,
        Query, QueryBinding, QueryBindings, QueryGuards, StructProps, TupleItems,
    },
    fn_def::FnDef,
    module_def::{ModuleDef, ModuleExports},
    type_expression::{TypeDef, TypeExpression},
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

    rule fn_def() -> AST
        = _ "let " _ id:identifier() _ args:(fn_def_arg())* _ "=" _ body:(expression()) _ {
            AST::FnDef(FnDef::new(id, FnDefArgs::new(args), Box::new(body)))
        }

    rule fn_def_arg() -> Identifier
        = _ id:identifier() _ {
            id
        }

    rule type_def() -> AST
        = _ "type " _ id:identifier() _ "::" _ t_expr:type_expr() {
            AST::TypeDef(TypeDef::new(id, Box::new(t_expr)))
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
        / fn_pipe()
        / query()
        / lambda()
        / fn_call()
        / op_call()
        / sum()
        / variable()
        / literal_expr()
        / commented_expr()

    rule fn_pipe() -> Expression
        = a:fn_pipe_arg() calls:(fn_pipe_call())+ {
            let call = calls.iter().fold(a, |acc, call|{
                if let Expression::FnCall(id, args) = call {
                    let mut args = args.clone();
                    args.prepend_arg(acc);
                    return Expression::FnCall(id.clone(), args)
                }
                panic!("Failed to create fn call pipeline")
            });

            call
        }

    rule fn_pipe_arg() -> Expression
        = lambda()
        / sum()
        / fn_call()
        / op_call()
        / atom()

    rule fn_pipe_call() -> Expression
        = _ "|>" _ fc:fn_call() {
            fc
        }
        / _ "|>" _ id:variable_identifier() {
            Expression::FnCall(id, FnCallArgs::new(vec![]))
        }

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
        = edge_prop()
        / variable()
        / literal_expr()
        / constant_or_type_ref()
        / "(" _ v:sum() _ ")" { v }
        / "(" _ l:lambda() _ ")" { l }
        / "(" _ c:(fn_pipe() / fn_call() / op_call()) _ ")" { c }


    rule variable() -> Expression
        = id:variable_identifier() {
            Expression::Var(id)
        }
        / "." id:variable_identifier() {
            Expression::PropFnRef(id)
        }

    rule query() -> Expression
        = bindings:query_binding()+ guards:query_guard()* _ prod:query_production() {
            Expression::Query(
                Query::new(
                    QueryBindings::new(bindings),
                    QueryGuards::new(guards),
                    Box::new(prod)
                )
            )
        }

    rule query_binding() -> QueryBinding
        = _ "?!" _ vars:query_binding_vars() _ "<-" _ expr:query_expr() _ {
            QueryBinding::new_negated(vars, Box::new(expr))
        }
        / _ "?" _ vars:query_binding_vars() _ "<-" _ expr:query_expr() _ {
            QueryBinding::new(vars, Box::new(expr))
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
        / "(" _ v:sum() _ ")" { v }
        / "(" _ l:lambda() _ ")" { l }
        / "(" _ c:(fn_pipe() / fn_call() / op_call()) _ ")" { c }
        / fn_call()
        / constant_or_type_ref()
        / lambda()
        / variable()
        / op_call()
        / sum()
        / literal_expr()

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
        / "!> " _ expr:query_expr() _ {
            expr
        }


    rule fn_call() -> Expression
        = _ id:identifier() args:(fn_arg())+ _ {
            let args = FnCallArgs::new(args);
            Expression::FnCall(id, args)
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
        / edge_prop()
        / variable()

    rule let_expr() -> Expression
        = "let" _ bindings:let_bindings() _ "in" _ body:let_body() {
            Expression::Let(
                LetExpression::new(LetBindings::new(bindings), Box::new(body))
            )
        }

    rule let_bindings() -> Vec<(Identifier, Expression)>
        = binding:let_binding() more_bindings:(additional_let_binding())* {
            let mut bindings = more_bindings;
            bindings.insert(0, binding);
            bindings
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

    rule let_body() -> Expression
        = lambda()
        / fn_pipe()
        / fn_call()
        / op_call()
        / sum()
        / variable()
        / literal_expr()
        / commented_let_body()
        / query()

    rule commented_let_body() -> Expression
        = c:comment() _ body:let_body() {
            Expression::Commented(c, Box::new(body))
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
        / "[" _ comment() _ "]" {
            Expression::Lit(Literal::ListLit(TupleItems::from(vec![])))
        }

    rule tuple_item() -> Expression
        = fn_pipe()
        / fn_call()
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
        = args:lambda_args() " "+ "->" _ body:let_body() {
            Expression::Lambda(Lambda::new(LambdaArgs::new(args), Box::new(body)))
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
        = "!" id:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '!' | '^' | '=' | '|'])+ {
            join_string("!", id)
        }
        / id:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '^' | '=' | '|'])+ id2:$(['+' | '-' | '*' | '/' | '>' | '<' | '=' | '!' | '^' | '=' | '|'])* {
            join_string(String::from_iter(id).as_str(), id2)
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

pub fn parse(str: &str) -> ParseResult {
    parser::program(str)
}

#[cfg(test)]
pub type ParseASTResult = Result<Box<AST>, ParseError<LineCol>>;

#[cfg(test)]
pub fn parse_ast(str: &str) -> ParseASTResult {
    parser::program_root_def(str)
}

#[cfg(test)]
pub type ParseExprResult = Result<Box<Expression>, ParseError<LineCol>>;

#[cfg(test)]
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
