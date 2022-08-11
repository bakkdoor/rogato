use fn_call::FnCallArgs;
use let_expression::LetExpression;

use super::{
    expression::{Lambda, Literal, Query},
    *,
};

pub trait Visitor<T: Default> {
    fn root_comment(&mut self, _comment: &str) -> T {
        T::default()
    }
    fn module_def(&mut self, _mod_def: &ModuleDef) -> T {
        T::default()
    }
    fn use_stmt(&mut self, _mod_def: &Identifier) -> T {
        T::default()
    }
    fn fn_def(&mut self, _fn_def: &FnDef) -> T {
        T::default()
    }
    fn type_def(&mut self, _type_def: &TypeDef) -> T {
        T::default()
    }

    fn commented(&mut self, _commented: &str, _expr: &Expression) -> T {
        T::default()
    }
    fn lit(&mut self, _lit: &Literal) -> T {
        T::default()
    }
    fn fn_call(&mut self, _id: &Identifier, _args: &FnCallArgs) -> T {
        T::default()
    }
    fn op_call(&mut self, _id: &Identifier, _left: &Expression, _right: &Expression) -> T {
        T::default()
    }
    fn var(&mut self, _id: &Identifier) -> T {
        T::default()
    }
    fn const_or_type_ref(&mut self, _id: &Identifier) -> T {
        T::default()
    }
    fn prop_fn_ref(&mut self, _id: &Identifier) -> T {
        T::default()
    }
    fn edge_prop(&mut self, _expr: &Expression, _edge: &Identifier) -> T {
        T::default()
    }
    fn let_(&mut self, _let_expr: &LetExpression) -> T {
        T::default()
    }
    fn lambda(&mut self, _lambda: &Lambda) -> T {
        T::default()
    }
    fn query(&mut self, _query: &Query) -> T {
        T::default()
    }
    fn symbol(&mut self, _id: &Identifier) -> T {
        T::default()
    }
    fn quoted(&mut self, _expr: &Expression) -> T {
        T::default()
    }
    fn quoted_ast(&mut self, _ast: &AST) -> T {
        T::default()
    }
    fn unquoted(&mut self, _expr: &Expression) -> T {
        T::default()
    }
    fn unquoted_ast(&mut self, _ast: &AST) -> T {
        T::default()
    }
    fn inline_fn_def(&mut self, _fn_def: &FnDef) -> T {
        T::default()
    }
}
