use rogato_common::{
    ast::module_def::ModuleDef,
    val::{self, ValueRef},
};

use crate::{module::Module, EvalContext, EvalError, Evaluate};

impl Evaluate<ValueRef> for ModuleDef {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match context.lookup_module(self.id()) {
            Some(mut module) => {
                for eid in self.exports().iter() {
                    module.export(eid.clone());
                }
                context.set_current_module(self.id().clone())
            }
            None => {
                let mut module = Module::new(self.id());
                for eid in self.exports().iter() {
                    module.export(eid.clone());
                }
                context.define_module(module);
                context.set_current_module(self.id().clone());
            }
        }

        Ok(val::object([
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
