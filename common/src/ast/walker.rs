use super::{
    expression::{ExprKind, Expression, Literal},
    fn_def::FnDefVariant,
    visitor::Visitor,
    AST,
};

pub trait Walk<T: Default = ()> {
    fn walk<V: Visitor<T>>(&self, v: &mut V);
}

impl Walk for AST {
    fn walk<V: Visitor<()>>(&self, v: &mut V) {
        match self {
            AST::RootComment(c) => v.root_comment(c),
            AST::ModuleDef(mod_def) => v.module_def(mod_def),
            AST::Use(id, imports) => v.use_stmt(id, imports),
            AST::FnDef(fn_def) => {
                let fn_def = fn_def.borrow();
                v.fn_def(&fn_def);
                for FnDefVariant(args, body, _) in fn_def.variants_iter() {
                    for a in args.iter() {
                        a.walk(v);
                    }
                    body.walk(v);
                }
            }
            AST::TypeDef(type_def) => v.type_def(type_def),
        }
    }
}

impl Walk for Expression {
    fn walk<V: Visitor<()>>(&self, v: &mut V) {
        match &self.kind {
            ExprKind::Commented(c, expr) => {
                v.commented(c, expr);
                expr.walk(v);
            }
            ExprKind::Lit(lit_exp) => {
                v.lit(lit_exp);
                match lit_exp {
                    Literal::Bool(_) => {}
                    Literal::Number(_) => {}
                    Literal::String(_) => {}
                    Literal::List(vals) => {
                        for val in vals.iter() {
                            val.walk(v)
                        }
                    }
                    Literal::ListCons(first, rest) => {
                        first.walk(v);
                        rest.walk(v)
                    }
                    Literal::Tuple(vals) => {
                        for val in vals.iter() {
                            val.walk(v)
                        }
                    }
                    Literal::Struct(_id, props) => {
                        for (_prop_name, val) in props.iter() {
                            val.walk(v)
                        }
                    }
                    Literal::Map(kv_pairs) => {
                        for kv_pair in kv_pairs.iter() {
                            kv_pair.key.walk(v);
                            kv_pair.value.walk(v)
                        }
                    }
                    Literal::MapCons(kv_pairs, rest) => {
                        for kv_pair in kv_pairs.iter() {
                            kv_pair.key.walk(v);
                            kv_pair.value.walk(v)
                        }
                        rest.walk(v)
                    }
                }
            }
            ExprKind::FnCall(fn_call) => {
                v.fn_call(fn_call);
                for a in fn_call.args.iter() {
                    a.walk(v);
                }
            }
            ExprKind::OpCall(id, left, right) => {
                v.op_call(id, left, right);
                left.walk(v);
                right.walk(v);
            }
            ExprKind::Var(id) => v.var(id),
            ExprKind::ConstOrTypeRef(id) => v.const_or_type_ref(id),
            ExprKind::DBTypeRef(id) => v.db_type_ref(id),
            ExprKind::PropFnRef(id) => v.prop_fn_ref(id),
            ExprKind::EdgeProp(id, edge) => v.edge_prop(id, edge),
            ExprKind::IfElse(if_else) => if_else.walk(v),
            ExprKind::Let(let_expr) => let_expr.walk(v),
            ExprKind::Lambda(lambda) => lambda.walk(v),
            ExprKind::Query(query) => query.walk(v),
            ExprKind::Symbol(id) => v.symbol(id),
            ExprKind::Quoted(expr) => v.quoted(expr),
            ExprKind::QuotedAST(ast) => v.quoted_ast(ast),
            ExprKind::Unquoted(expr) => v.unquoted(expr),
            ExprKind::UnquotedAST(ast) => v.unquoted_ast(ast),
            ExprKind::InlineFnDef(fn_def) => v.inline_fn_def(&fn_def.borrow()),
        }
    }
}
