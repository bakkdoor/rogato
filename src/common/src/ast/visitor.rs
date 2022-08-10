use fn_call::FnCallArgs;
use let_expression::LetExpression;

use super::{
    expression::{Lambda, Literal, Query},
    *,
};

pub trait Visitor {
    fn root_comment(&mut self, _comment: &str) {}
    fn module_def(&mut self, _mod_def: &ModuleDef) {}
    fn use_stmt(&mut self, _mod_def: &Identifier) {}
    fn fn_def(&mut self, _fn_def: &FnDef) {}
    fn type_def(&mut self, _type_def: &TypeDef) {}

    fn commented(&mut self, _commented: &str, _expr: &Expression) {}
    fn lit(&mut self, _lit: &Literal) {}
    fn fn_call(&mut self, _id: &Identifier, _args: &FnCallArgs) {}
    fn op_call(&mut self, _id: &Identifier, _left: &Expression, _right: &Expression) {}
    fn var(&mut self, _id: &Identifier) {}
    fn const_or_type_ref(&mut self, _id: &Identifier) {}
    fn prop_fn_ref(&mut self, _id: &Identifier) {}
    fn edge_prop(&mut self, _expr: &Expression, _edge: &Identifier) {}
    fn let_(&mut self, _let_expr: &LetExpression) {}
    fn lambda(&mut self, _lambda: &Lambda) {}
    fn query(&mut self, _query: &Query) {}
    fn symbol(&mut self, _id: &Identifier) {}
    fn quoted(&mut self, _expr: &Expression) {}
    fn quoted_ast(&mut self, _ast: &AST) {}
    fn unquoted(&mut self, _expr: &Expression) {}
    fn unquoted_ast(&mut self, _ast: &AST) {}
}
