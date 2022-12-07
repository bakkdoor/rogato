use rogato_common::{
    ast::Program,
    val::{self, ValueRef},
};

use crate::{EvalContext, EvalError, Evaluate};

#[cfg(feature = "flame_it")]
use flamer::flame;

impl Evaluate<ValueRef> for Program {
    #[cfg_attr(feature = "flame_it", flame("Programm::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut values = Vec::with_capacity(self.len());
        for ast in self.iter() {
            values.push(ast.evaluate(context)?)
        }
        Ok(val::list(values))
    }
}
