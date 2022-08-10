use rogato_common::{ast::fn_call::FnCallArgs, val::ValueRef};

use crate::eval::{EvalContext, EvalError, Evaluate};

impl Evaluate<Vec<ValueRef>> for FnCallArgs {
    fn evaluate(&self, context: &mut EvalContext) -> Result<Vec<ValueRef>, EvalError> {
        let mut values = vec![];
        for arg in self.iter() {
            match arg.evaluate(context) {
                Ok(val) => values.push(val),
                Err(e) => return Err(EvalError::FnCallArgumentError(Box::new(e))),
            }
        }
        Ok(values)
    }
}
