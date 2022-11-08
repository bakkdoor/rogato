use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::{expression::Expression, let_expression::LetExpression},
    val::ValueRef,
};

impl Evaluate<ValueRef> for LetExpression {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut context = context.with_child_env();

        for (id, expr) in self.bindings.iter() {
            match &**expr {
                Expression::InlineFnDef(fn_def) => {
                    context.define_fn(Rc::clone(fn_def));
                }
                _ => match expr.evaluate(&mut context) {
                    Ok(val) => context.define_var(id, val),
                    Err(e) => return Err(e),
                },
            }
        }

        self.body.evaluate(&mut context)
    }
}
