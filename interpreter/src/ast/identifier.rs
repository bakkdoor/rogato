use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{ast::VarIdentifier, val::ValueRef};

#[cfg(feature = "flame_it")]
use flamer::flame;

impl Evaluate<ValueRef> for VarIdentifier {
    #[cfg_attr(feature = "flame_it", flame("VarIdentifier::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match context.lookup_var(self) {
            Some(val) => Ok(val),
            None => Err(EvalError::VarNotDefined(self.clone())),
        }
    }
}
