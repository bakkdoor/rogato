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

    pub rule program_root_def() -> AST
        = _ def:root_def() {
            def
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
        = "use " id:constant_or_type_ref() {
            AST::Use(id.to_string())
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
        = fn_pipe()
        / tuple_lit()
        / list_lit()
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
                    args.prepend_arg(acc);
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
            Expression::Commented(c, Box::new(e))
        }

    rule atom() -> Expression
        = literal_expr()
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
            Expression::Quoted(Box::new(expr))
        }
        / "^" "(" ast:root_def() ")" {
            Expression::QuotedAST(Box::new(ast))
        }
        / symbol()
        / unquoted_expr()

    rule unquoted_expr() -> Expression
        = "~" "(" expr:expression() ")" {
            Expression::Unquoted(Box::new(expr))
        }
        / "~" "(" ast:root_def() ")" {
            Expression::UnquotedAST(Box::new(ast))
        }
        / "~" var:variable() {
            Expression::Unquoted(Box::new(var))
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
                    Box::new(prod)
                )
            )
        }

    rule query_binding() -> QueryBinding
        = _ "?" _ vars:query_binding_vars() _ "<!-" _ expr:query_expr() _ {
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
        / "(" _ l:lambda() _ ")" { l }
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
            Expression::EdgeProp(Box::new(expr), edge)
        }

    rule edge_prop_expr() -> Expression
        = variable()
        / "(" _ c:(fn_pipe() / fn_call() / op_call()) _ ")" { c }

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
        / atom()

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
        / query()
        / fn_pipe()
        / fn_call()
        / op_call()
        / atom()
        / commented_let_body()

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
            Expression::Lit(Literal::Int64(n.parse().unwrap()))
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
            Expression::Commented(c, Box::new(item))
        }

    rule additional_tuple_item() -> Expression
        = _ "," _ item:tuple_item() {
            item
        }

    rule struct_lit() -> Expression
        = id:struct_identifier() "{" _ first:struct_prop() rest:(additional_struct_prop())*  _ ("," _)? "}" {
            Expression::Lit(Literal::Struct(id, Box::new(StructProps::new(first, rest))))
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

    rule symbol_identifier() -> Identifier
        = variable_identifier()
        / struct_identifier()
        / identifier()

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

#[allow(dead_code)]
pub fn parse_traced(str: &str) -> ParseResult {
    parser::traced_program(str)
}

#[cfg(test)]
pub type ParseASTResult = Result<Box<AST>, ParseError<LineCol>>;

#[cfg(test)]
pub fn parse_ast(str: &str) -> ParseASTResult {
    match parser::program_root_def(str) {
        Ok(program) => Ok(Box::new(program)),
        Err(e) => Err(e),
    }
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
