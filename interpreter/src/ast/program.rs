use rogato_common::{
    ast::Program,
    val::{self, ValueRef},
};

use crate::{EvalContext, EvalError, Evaluate};

impl Evaluate<ValueRef> for Program {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut values = vec![];
        for ast in self.iter() {
            values.push(ast.evaluate(context)?)
        }
        Ok(val::list(values))
    }
}
