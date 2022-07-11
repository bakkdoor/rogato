use super::{expression::Expression, visitor::Visitor, AST};

pub trait Walk {
    fn walk<V: Visitor>(&self, v: &mut V);
}

impl Walk for AST {
    fn walk<V: Visitor>(&self, v: &mut V) {
        match self {
            AST::RootComment(c) => v.root_comment(c),
            AST::ModuleDef(mod_def) => v.module_def(mod_def),
            AST::FnDef(fn_def) => {
                v.fn_def(fn_def);
                fn_def.body().walk(v)
            }
            AST::TypeDef(type_def) => v.type_def(type_def),
        }
    }
}

impl Walk for Expression {
    fn walk<V: Visitor>(&self, v: &mut V) {
        match self {
            Expression::Commented(c, expr) => {
                v.commented(c, expr);
                expr.walk(v);
            }
            Expression::Lit(lit_exp) => v.lit(lit_exp),
            Expression::Sum(a, b) => {
                v.sum(a, b);
                a.walk(v);
                b.walk(v);
            }
            Expression::Product(a, b) => {
                v.product(a, b);
                a.walk(v);
                b.walk(v);
            }
            Expression::FnCall(id, args) => {
                v.fn_call(id, args);
                for a in args.iter() {
                    a.walk(v);
                }
            }
            Expression::OpCall(id, left, right) => {
                v.op_call(id, left, right);
                left.walk(v);
                right.walk(v);
            }
            Expression::Var(id) => v.var(id),
            Expression::ConstOrTypeRef(id) => v.const_or_type_ref(id),
            Expression::PropFnRef(id) => v.prop_fn_ref(id),
            Expression::EdgeProp(id, edge) => v.edge_prop(id, edge),
            Expression::Let(bindings, body) => {
                v.let_(bindings, body);
                body.walk(v);
                for (_id, val) in bindings.iter() {
                    val.walk(v);
                }
            }
            Expression::Lambda(lambda) => {
                v.lambda(lambda);
                lambda.body().walk(v);
            }
            Expression::Query(query) => {
                v.query(query);
                for binding in query.bindings().iter() {
                    binding.value().walk(v);
                }
                for g in query.guards().iter() {
                    g.walk(v);
                }
                query.production().walk(v);
            }
        }
    }
}
