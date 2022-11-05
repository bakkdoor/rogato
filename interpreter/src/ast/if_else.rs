use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::ast::if_else::IfElse;
use rogato_common::val::{Value, ValueRef};

impl Evaluate<ValueRef> for IfElse {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let val = self.condition.evaluate(context)?;
        match *val {
            Value::Bool(true) => self.then_expr.evaluate(context),
            Value::Bool(false) => self.else_expr.evaluate(context),
            _ => Err(EvalError::IFElseConditionNotBool(Rc::clone(&val))),
        }
    }
}
