use super::{expression::Expression, visitor::Visitor, AST};

pub trait Walk {
    fn walk<V: Visitor<()>>(&self, v: &mut V);
}

impl Walk for AST {
    fn walk<V: Visitor<()>>(&self, v: &mut V) {
        match self {
            AST::RootComment(c) => v.root_comment(c),
            AST::ModuleDef(mod_def) => v.module_def(mod_def),
            AST::Use(id) => v.use_stmt(id),
            AST::FnDef(fn_def) => {
                v.fn_def(fn_def);
                fn_def.body().walk(v)
            }
            AST::TypeDef(type_def) => v.type_def(type_def),
        }
    }
}

impl Walk for Expression {
    fn walk<V: Visitor<()>>(&self, v: &mut V) {
        match self {
            Expression::Commented(c, expr) => {
                v.commented(c, expr);
                expr.walk(v);
            }
            Expression::Lit(lit_exp) => v.lit(lit_exp),
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
            Expression::Let(let_expr) => let_expr.walk(v),
            Expression::Lambda(lambda) => lambda.walk(v),
            Expression::Query(query) => query.walk(v),
            Expression::Symbol(id) => v.symbol(id),
            Expression::Quoted(expr) => v.quoted(expr),
            Expression::QuotedAST(ast) => v.quoted_ast(ast),
            Expression::Unquoted(expr) => v.unquoted(expr),
            Expression::UnquotedAST(ast) => v.unquoted_ast(ast),
            Expression::InlineFnDef(fn_def) => v.inline_fn_def(fn_def),
        }
    }
}
