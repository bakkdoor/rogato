use fn_call::FnCallArgs;

use super::{
    expression::{LambdaArgs, LetBindings, Literal},
    *,
};

pub trait Visitor {
    fn program(&mut self, program: &Program) {
        for node in program.iter() {
            match &**node {
                AST::RootComment(c) => self.root_comment(&c),
                AST::ModuleDef(id, exports) => self.module_def(&id, &exports),
                AST::FnDef(id, args, body) => self.fn_def(&id, &args, &body),
                AST::TypeDef(id, type_expr) => self.type_def(&id, &type_expr),
            }
        }
    }

    fn root_comment(&mut self, _comment: &String) {}
    fn module_def(&mut self, _id: &Identifier, _exports: &ModuleExports) {}
    fn fn_def(&mut self, _id: &Identifier, _args: &FnDefArgs, _body: &Box<Expression>) {}
    fn type_def(&mut self, _id: &Identifier, _type_expr: &Box<TypeExpression>) {}

    fn expression(&mut self, _expr: &Expression) {}
    fn commented(&mut self, _commented: &String, _expr: &Box<Expression>) {}
    fn lit(&mut self, _lit: &Literal) {}
    fn sum(&mut self, _left: &Box<Expression>, _right: &Box<Expression>) {}
    fn product(&mut self, _left: &Box<Expression>, _right: &Box<Expression>) {}
    fn fn_call(&mut self, _id: &Identifier, _args: &Box<FnCallArgs>) {}
    fn op_call(&mut self, _id: &Identifier, _left: &Box<Expression>, _right: &Box<Expression>) {}
    fn var(&mut self, _id: &Identifier) {}
    fn const_or_type_ref(&mut self, _id: &Identifier) {}
    fn let_(&mut self, _bindings: &Box<LetBindings>, _expr: &Box<Expression>) {}
    fn lambda(&mut self, _args: &Box<LambdaArgs<Identifier>>, _body: &Box<Expression>) {}
}
