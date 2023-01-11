use rogato_common::{
    ast::module_def::ModuleDef,
    val::{self, ValueRef},
};

use crate::{environment::Imports, module::Module, EvalContext, EvalError, Evaluate};

#[cfg(feature = "flame_it")]
use flamer::flame;

impl Evaluate<ValueRef> for ModuleDef {
    #[cfg_attr(feature = "flame_it", flame("ModuleDef::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match context.lookup_module(self.id()) {
            Some(mut module) => {
                module.export(self.exports());
                context.set_current_module(self.id().clone());
                context.import(self.id(), Imports::All)?;
            }
            None => {
                let mut module = Module::new(self.id());
                module.export(self.exports());
                context.define_module(module);
                context.set_current_module(self.id().clone());
                context.import(self.id(), Imports::All)?;
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
