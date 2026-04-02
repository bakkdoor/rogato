use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::fn_def::{FnDef, FnDefVariant},
    val,
    val::ValueRef,
};

#[cfg(feature = "flame_it")]
use flamer::flame;

impl Evaluate<ValueRef> for FnDef {
    #[cfg_attr(feature = "flame_it", flame("FnDef::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        for FnDefVariant(args, body, return_type) in self.variants_iter() {
            let fn_def_variant = match return_type {
                Some(rt) => rogato_common::ast::fn_def::FnDefVariant::new_with_return_type(
                    args.clone(),
                    Rc::clone(body),
                    Rc::clone(rt),
                ),
                None => {
                    rogato_common::ast::fn_def::FnDefVariant::new(args.clone(), Rc::clone(body))
                }
            };
            context.define_fn(self.id(), fn_def_variant);
        }
        Ok(val::symbol(self.id().clone()))
    }
}
