use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{ast::fn_def::FnDef, val, val::ValueRef};

impl Evaluate<ValueRef> for FnDef {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        for (args, body) in self.variants.iter() {
            let fn_def = FnDef::new(self.id().clone(), args.clone(), Rc::clone(body));
            context.define_fn(fn_def);
        }
        Ok(val::symbol(self.id().clone()))
    }
}
