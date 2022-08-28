use std::ops::Deref;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::ast::if_else::IfElse;
use rogato_common::val::{Value, ValueRef};

impl Evaluate<ValueRef> for IfElse {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match self.condition().evaluate(context)?.deref() {
            Value::Bool(true) => self.then_expr().evaluate(context),
            Value::Bool(false) => self.else_expr().evaluate(context),
            val => Err(EvalError::Unknown(format!(
                "IFElse condition is not a boolean: {:?}",
                val
            ))),
        }
    }
}
