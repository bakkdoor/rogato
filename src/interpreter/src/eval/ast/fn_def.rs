use crate::eval::{EvalContext, EvalError, Evaluate};
use rogato_common::{ast::fn_def::FnDef, val, val::ValueRef};

impl Evaluate<ValueRef> for FnDef {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        context.define_fn(FnDef::new(
            self.id().clone(),
            self.args().clone(),
            self.body(),
        ));
        Ok(val::object(vec![
            ("type", val::string("Fn")),
            ("name", val::string(self.id().to_string())),
            (
                "args",
                val::list(
                    self.args()
                        .iter()
                        .map(val::string)
                        .collect::<Vec<ValueRef>>(),
                ),
            ),
        ]))
    }
}
