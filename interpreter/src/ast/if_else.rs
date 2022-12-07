use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::ast::if_else::IfElse;
use rogato_common::val::{Value, ValueRef};

#[cfg(feature = "flame_it")]
use flamer::flame;

impl Evaluate<ValueRef> for IfElse {
    #[cfg_attr(feature = "flame_it", flame("IfElse::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let val = self.condition.evaluate(context)?;
        match *val {
            Value::Bool(true) => self.then_expr.evaluate(context),
            Value::Bool(false) => self.else_expr.evaluate(context),
            _ => Err(EvalError::IFElseConditionNotBool(ValueRef::clone(&val))),
        }
    }
}
