use fn_call::FnCallArgs;

use super::{
    expression::{LambdaArgs, LetBindings, Literal, QueryBindings, QueryGuards},
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
    fn fn_def(&mut self, _id: &Identifier, _args: &FnDefArgs, _body: &Expression) {}
    fn type_def(&mut self, _id: &Identifier, _type_expr: &TypeExpression) {}

    fn expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Commented(c, e) => self.commented(&c, &e),
            Expression::Lit(l) => self.lit(&l),
            Expression::Sum(left, right) => self.sum(&left, &right),
            Expression::Product(left, right) => self.product(&left, &right),
            Expression::FnCall(id, args) => self.fn_call(&id, &args),
            Expression::OpCall(id, left, right) => self.op_call(&id, &left, &right),
            Expression::Var(id) => self.var(&id),
            Expression::ConstOrTypeRef(id) => self.const_or_type_ref(&id),
            Expression::EdgeProp(id, edge) => self.edge_prop(&id, &edge),
            Expression::Let(bindings, expr) => self.let_(&bindings, &expr),
            Expression::Lambda(args, body) => self.lambda(&args, &body),
            Expression::Query(query, guards, production) => {
                self.query(&query, &guards, &production)
            }
        }
    }

    fn commented(&mut self, _commented: &String, _expr: &Expression) {}
    fn lit(&mut self, _lit: &Literal) {}
    fn sum(&mut self, _left: &Expression, _right: &Expression) {}
    fn product(&mut self, _left: &Expression, _right: &Expression) {}
    fn fn_call(&mut self, _id: &Identifier, _args: &FnCallArgs) {}
    fn op_call(&mut self, _id: &Identifier, _left: &Expression, _right: &Expression) {}
    fn var(&mut self, _id: &Identifier) {}
    fn const_or_type_ref(&mut self, _id: &Identifier) {}
    fn edge_prop(&mut self, _expr: &Expression, _edge: &Identifier) {}
    fn let_(&mut self, _bindings: &LetBindings, _expr: &Expression) {}
    fn lambda(&mut self, _args: &LambdaArgs<Identifier>, _body: &Expression) {}
    fn query(
        &mut self,
        _bindings: &QueryBindings,
        _guards: &QueryGuards,
        _production: &Expression,
    ) {
    }
}
