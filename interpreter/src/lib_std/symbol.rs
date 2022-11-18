use super::invalid_args;
use crate::module::Module;
use rogato_common::{
    ast::module_def::ModuleExports,
    val::{self, Value},
};
use std::ops::Deref;

pub fn module() -> Module {
    let mut module = Module::new("Std.Symbol");
    module.export(&ModuleExports::new(vec![
        "toString".into(),
        "fromString".into(),
    ]));

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

    module.fn_def_native("fromString", &["string"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Symbol.fromString"));

        match (args.len(), args.get(0)) {
            (1, Some(string)) => match string.deref() {
                Value::String(string) => Ok(val::symbol(string.trim())),
                _ => error,
            },

            (_, _) => error,
        }
    });

    module
}
