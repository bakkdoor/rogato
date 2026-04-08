use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::{
        expression::{ExprKind, Expression},
        helpers::{fn_call, lambda, var},
    },
    val::ValueRef,
    val::{self},
};

#[cfg(feature = "flame_it")]
use flamer::flame;

impl Evaluate<ValueRef> for Expression {
    #[cfg_attr(feature = "flame_it", flame("Expression::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match &self.kind {
            ExprKind::Commented(_c, e) => e.evaluate(context),
            ExprKind::Lit(lit_exp) => lit_exp.evaluate(context),
            ExprKind::FnCall(fn_call) => fn_call.evaluate(context),
            ExprKind::OpCall(op_ident, left, right) => {
                let call_args = [left.evaluate(context)?, right.evaluate(context)?];
                match context.call_function(op_ident, &call_args) {
                    Some(val) => Ok(val?),
                    None => Err(EvalError::OperatorNotDefined(op_ident.clone())),
                }
            }
            ExprKind::Var(id) => match context.lookup_var(id) {
                Some(var) => Ok(var),
                None => match context.call_function(&id.into(), &[]) {
                    Some(val) => Ok(val?),
                    None => Err(EvalError::VarNotDefined(id.clone())),
                },
            },
            ExprKind::ConstOrTypeRef(id) => match context.lookup_const(id) {
                Some(val) => Ok(val),
                None => match context.lookup_type(id) {
                    Some(type_) => Ok(val::object([
                        ("type", val::string("TypeExpression")),
                        ("id", val::string(type_.id())),
                        ("expression", val::string(format!("{type_}"))),
                    ])),
                    None => Err(EvalError::ConstOrTypeNotFound(id.clone())),
                },
            },
            ExprKind::DBTypeRef(id) => match context.lookup_db_type(id) {
                Some(type_) => Ok(val::object([
                    ("type", val::string("DBType")),
                    ("id", val::string(type_.id())),
                    ("expression", val::string(format!("{type_}"))),
                ])),
                None => Err(EvalError::DBTypeNotFound(id.clone())),
            },
            ExprKind::PropFnRef(id) => {
                let lambda = lambda(["object"], fn_call(id, [var("object")]));
                lambda.evaluate(context)
            }
            ExprKind::EdgeProp(_id, _edge) => Ok(val::string("eval edge prop")),
            ExprKind::IfElse(if_else) => if_else.evaluate(context),
            ExprKind::Let(let_expr) => let_expr.evaluate(context),
            ExprKind::Lambda(lambda) => lambda.evaluate(context),
            ExprKind::Query(query) => query.evaluate(context),
            ExprKind::Symbol(id) => Ok(val::symbol(id.clone())),
            ExprKind::Quoted(expr) => Ok(val::quoted(Rc::clone(expr))),
            ExprKind::QuotedAST(ast) => Ok(val::quoted_ast(Rc::clone(ast))),
            ExprKind::Unquoted(expr) => Ok(val::string(format!("~({expr})"))),
            ExprKind::UnquotedAST(ast) => Ok(val::string(format!("~({ast})"))),
            ExprKind::InlineFnDef(fn_def) => fn_def.borrow().evaluate(context),
        }
    }
}
