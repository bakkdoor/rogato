use rogato_common::{val::{ValueRef, self}, ast::Program};

use crate::eval::{Evaluate, EvalContext, EvalError};

impl Evaluate<ValueRef> for Program {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut values = vec![];
        for ast in self.iter() {
            values.push(ast.evaluate(context)?)
        }
        Ok(val::list(values))
    }
}
