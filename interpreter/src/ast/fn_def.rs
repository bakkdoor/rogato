use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{ast::fn_def::FnDef, val, val::ValueRef};

impl Evaluate<ValueRef> for FnDef {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let fn_def = FnDef::new(self.id().clone(), self.args().clone(), self.body());
        context.define_fn(Rc::clone(&fn_def));
        Ok(val::fn_ref(fn_def))
    }
}
