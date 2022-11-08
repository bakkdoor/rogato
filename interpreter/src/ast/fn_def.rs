use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{ast::fn_def::FnDef, val, val::ValueRef};
use std::rc::Rc;

impl Evaluate<ValueRef> for FnDef {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match context.lookup_fn(self.id()) {
            Some(_func) => context.define_fn_variant(self.id(), self.args().clone(), self.body()),
            None => {
                let fn_def = FnDef::new(self.id().clone(), self.args().clone(), self.body());
                context.define_fn(Rc::clone(&fn_def));
            }
        }

        Ok(val::symbol(self.id().clone()))
    }
}
