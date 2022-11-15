use super::invalid_args;
use crate::module::Module;
use rogato_common::val::{self, Value};
use std::ops::Deref;

pub fn module() -> Module {
    let mut module = Module::new("Std.Symbol");

    module.fn_def_native("toString", &["symbol"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Symbol.toString"));

        match (args.len(), args.get(0)) {
            (1, Some(symbol)) => match symbol.deref() {
                Value::Symbol(symbol) => Ok(val::string(symbol)),
                _ => error,
            },

            (_, _) => error,
        }
    });

    module
}
