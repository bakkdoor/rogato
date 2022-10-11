use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::{
        expression::Expression,
        helpers::{fn_call, lambda, var},
    },
    val::ValueRef,
    val::{self, Value},
};

impl Evaluate<ValueRef> for Expression {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match self {
            Expression::Commented(_c, e) => e.evaluate(context),
            Expression::Lit(lit_exp) => lit_exp.evaluate(context),
            Expression::FnCall(fn_ident, args) => {
                let call_args = args.evaluate(context)?;
                match context.call_function(fn_ident, &call_args) {
                    Some(val) => Ok(val?),
                    None => match context.lookup_var(fn_ident) {
                        Some(val2) => match &*val2 {
                            Value::Lambda(lambda_ctx, lambda) => {
                                context.call_lambda(Rc::clone(lambda_ctx), lambda, &call_args)
                            }
                            Value::Symbol(fn_id) => {
                                context.call_function(fn_id, &call_args).unwrap_or_else(|| {
                                    Err(EvalError::FunctionNotDefined(fn_id.clone()))
                                })
                            }
                            _ => Err(EvalError::FunctionNotDefined(fn_ident.clone())),
                        },
                        None => Err(EvalError::FunctionNotDefined(fn_ident.clone())),
                    },
                }
            }
            Expression::OpCall(op_ident, left, right) => {
                let call_args = [left.evaluate(context)?, right.evaluate(context)?];
                match context.call_function(op_ident, &call_args) {
                    Some(val) => Ok(val?),
                    None => Err(EvalError::OperatorNotDefined(op_ident.clone())),
                }
            }
            Expression::Var(id) => match context.lookup_var(id) {
                Some(var) => Ok(var),
                None => match context.call_function(id, &[]) {
                    Some(val) => Ok(val?),
                    None => Err(EvalError::VarNotDefined(id.clone())),
                },
            },
            Expression::ConstOrTypeRef(id) => match context.lookup_const(id) {
                Some(val) => Ok(val),
                None => match context.lookup_type(id) {
                    Some(type_) => Ok(val::object([
                        ("type", val::string("TypeExpression")),
                        ("id", val::string(type_.id())),
                        ("expression", val::string(format!("{}", type_))),
                    ])),
                    None => Err(EvalError::ConstOrTypeNotFound(id.clone())),
                },
            },
            Expression::DBTypeRef(id) => match context.lookup_db_type(id) {
                Some(type_) => Ok(val::object([
                    ("type", val::string("DBType")),
                    ("id", val::string(type_.id())),
                    ("expression", val::string(format!("{}", type_))),
                ])),
                None => Err(EvalError::DBTypeNotFound(id.clone())),
            },
            Expression::PropFnRef(id) => {
                let lambda = lambda(["object"], fn_call(id, [var("object")]));
                lambda.evaluate(context)
            }
            Expression::EdgeProp(_id, _edge) => Ok(val::string("eval edge prop")),
            Expression::IfElse(if_else) => if_else.evaluate(context),
            Expression::Let(let_expr) => let_expr.evaluate(context),
            Expression::Lambda(lambda) => lambda.evaluate(context),
            Expression::Query(query) => query.evaluate(context),
            Expression::Symbol(id) => Ok(val::symbol(id.clone())),
            Expression::Quoted(expr) => Ok(val::quoted(Rc::clone(expr))),
            Expression::QuotedAST(ast) => Ok(val::quoted_ast(Rc::clone(ast))),
            Expression::Unquoted(expr) => Ok(val::string(format!("~({})", expr))),
            Expression::UnquotedAST(ast) => Ok(val::string(format!("~({})", ast))),
            Expression::InlineFnDef(fn_def) => fn_def.evaluate(context),
        }
    }
}
