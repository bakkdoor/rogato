use super::{expression::Expression, visitor::Visitor, AST};

#[allow(dead_code)]
pub(crate) fn walk<V: Visitor>(v: &mut V, ast: &AST) {
    match ast {
        AST::RootComment(c) => v.root_comment(c),
        AST::ModuleDef(id, exports) => v.module_def(id, exports),
        AST::FnDef(id, args, body) => {
            v.fn_def(id, args, body);
            walk_expr(v, body);
        }
        AST::TypeDef(id, type_expr) => v.type_def(id, type_expr),
    }
}

#[allow(dead_code)]
pub(crate) fn walk_expr<V: Visitor>(v: &mut V, expr: &Expression) {
    match expr {
        Expression::Commented(c, expr) => {
            v.commented(c, expr);
            walk_expr(v, expr)
        }
        Expression::Lit(lit_exp) => v.lit(lit_exp),
        Expression::Sum(a, b) => {
            v.sum(a, b);
            walk_expr(v, a);
            walk_expr(v, b);
        }
        Expression::Product(a, b) => {
            v.product(a, b);
            walk_expr(v, a);
            walk_expr(v, b);
        }
        Expression::FnCall(id, args) => {
            v.fn_call(id, args);
            for a in args.iter() {
                walk_expr(v, a);
            }
        }
        Expression::OpCall(id, left, right) => {
            v.op_call(id, left, right);
            walk_expr(v, left);
            walk_expr(v, right);
        }
        Expression::Var(id) => v.var(id),
        Expression::ConstOrTypeRef(id) => v.const_or_type_ref(id),
        Expression::EdgeProp(id, edge) => v.edge_prop(id, edge),
        Expression::Let(bindings, body) => {
            v.let_(bindings, body);
            walk_expr(v, body);
            for (_id, val) in bindings.iter() {
                walk_expr(v, val);
            }
        }
        Expression::Lambda(args, body) => {
            v.lambda(args, body);
            walk_expr(v, body);
        }
        Expression::Query(bindings, guards, production) => {
            v.query(bindings, guards, production);
            for binding in bindings.iter() {
                walk_expr(v, binding.value());
            }
            for g in guards.iter() {
                walk_expr(v, g);
            }
            walk_expr(v, production);
        }
    }
}
