use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{ast::let_expression::LetExpression, val::ValueRef};

impl Evaluate<ValueRef> for LetExpression {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        for (id, expr) in self.bindings().iter() {
            match expr.evaluate(context) {
                Ok(val) => context.define_var(id, val),
                Err(e) => return Err(e),
            }
        }
        self.body().evaluate(context)
    }
}
