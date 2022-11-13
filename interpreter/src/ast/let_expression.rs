use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::{expression::Expression, fn_def::FnDef, let_expression::LetExpression},
    val::ValueRef,
};

impl Evaluate<ValueRef> for LetExpression {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut context = context.with_child_env();

        for (id, expr) in self.bindings.iter() {
            match &**expr {
                Expression::InlineFnDef(fn_def) => {
                    let fn_def = fn_def.borrow();
                    for variant in fn_def.variants.iter() {
                        let fn_def = FnDef::new(
                            fn_def.id().clone(),
                            variant.0.clone(),
                            Rc::clone(&variant.1),
                        );
                        context.define_fn(fn_def);
                    }
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
