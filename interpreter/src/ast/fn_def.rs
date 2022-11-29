use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::fn_def::{FnDef, FnDefVariant},
    val,
    val::ValueRef,
};

impl Evaluate<ValueRef> for FnDef {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        for FnDefVariant(args, body) in self.variants.iter() {
            let fn_def_variant: FnDefVariant = FnDefVariant(args.clone(), Rc::clone(body));
            context.define_fn(self.id(), fn_def_variant);
        }
        Ok(val::symbol(self.id().clone()))
    }
}
