use rogato_common::{
    ast::module_def::ModuleDef,
    val::{self, ValueRef},
};

use crate::eval::{EvalContext, EvalError, Evaluate};

impl Evaluate<ValueRef> for ModuleDef {
    fn evaluate(&self, _context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        Ok(val::object(vec![
            ("type", val::string("Module")),
            ("name", val::string(self.id().clone())),
            (
                "exports",
                val::list(
                    self.exports()
                        .iter()
                        .map(|e| val::string(e.to_string()))
                        .collect::<Vec<ValueRef>>(),
                ),
            ),
        ]))
    }
}
