use super::invalid_args;
use crate::module::Module;
use rogato_common::{
    ast::module_def::ModuleExports,
    val::{self, Value},
};
use std::ops::Deref;

pub fn module() -> Module {
    let mut module = Module::new("Std.String");
    module.export(&ModuleExports::new(vec!["length".into()]));

    module.fn_def_native("length", &["string"], move |_ctx, args| {
        let error = Err(invalid_args("Std.String.length"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::String(string) => Ok(val::number(string.len())),
                _ => error,
            },
            _ => error,
        }
    });

    module
}
