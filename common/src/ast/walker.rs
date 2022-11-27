use super::{
    expression::{Expression, Literal},
    visitor::Visitor,
    AST,
};

pub trait Walk {
    fn walk<V: Visitor<()>>(&self, v: &mut V);
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
                for (args, body) in fn_def.variants.iter() {
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
        match self {
            Expression::Commented(c, expr) => {
                v.commented(c, expr);
                expr.walk(v);
            }
            Expression::Lit(lit_exp) => {
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
            Expression::FnCall(fn_call) => {
                v.fn_call(fn_call);
                for a in fn_call.args.iter() {
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
            Expression::DBTypeRef(id) => v.db_type_ref(id),
            Expression::PropFnRef(id) => v.prop_fn_ref(id),
            Expression::EdgeProp(id, edge) => v.edge_prop(id, edge),
            Expression::IfElse(if_else) => if_else.walk(v),
            Expression::Let(let_expr) => let_expr.walk(v),
            Expression::Lambda(lambda) => lambda.walk(v),
            Expression::Query(query) => query.walk(v),
            Expression::Symbol(id) => v.symbol(id),
            Expression::Quoted(expr) => v.quoted(expr),
            Expression::QuotedAST(ast) => v.quoted_ast(ast),
            Expression::Unquoted(expr) => v.unquoted(expr),
            Expression::UnquotedAST(ast) => v.unquoted_ast(ast),
            Expression::InlineFnDef(fn_def) => v.inline_fn_def(&fn_def.borrow()),
        }
    }
}
