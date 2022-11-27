use std::rc::Rc;

use rogato_common::{
    ast::fn_call::{FnCall, FnCallArgs},
    val::{Value, ValueRef},
};

use crate::{EvalContext, EvalError, Evaluate};

impl Evaluate<ValueRef> for FnCall {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let fn_ident = &self.id;
        let args = &self.args;
        let call_args = args.evaluate(context)?;
        match context.call_function(fn_ident, &call_args) {
            Some(val) => Ok(val?),
            None => match context.lookup_var(fn_ident) {
                Some(val2) => match &*val2 {
                    Value::Lambda(lambda_ctx, lambda) => {
                        context.call_lambda(Rc::clone(lambda_ctx), lambda, &call_args)
                    }
                    Value::Symbol(fn_id) => context
                        .call_function(fn_id, &call_args)
                        .unwrap_or_else(|| Err(EvalError::FunctionNotDefined(fn_id.clone()))),
                    _ => Err(EvalError::FunctionNotDefined(fn_ident.clone())),
                },
                None => Err(EvalError::FunctionNotDefined(fn_ident.clone())),
            },
        }
    }
}

impl Evaluate<Vec<ValueRef>> for FnCallArgs {
    fn evaluate(&self, context: &mut EvalContext) -> Result<Vec<ValueRef>, EvalError> {
        let mut values = Vec::with_capacity(self.len());
        for arg in self.iter() {
            match arg.evaluate(context) {
                Ok(val) => values.push(val),
                Err(e) => return Err(EvalError::FnCallArgumentError(Box::new(e))),
            }
        }
        Ok(values)
    }
}
