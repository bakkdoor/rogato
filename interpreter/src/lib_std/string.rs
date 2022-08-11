use super::{fn_def, invalid_args};
use crate::module::Module;
use rogato_common::val::{self, Value};
use std::ops::Deref;

pub fn module() -> Module {
    let mut module = Module::new("Std.String");

    module.fn_def(fn_def("length", vec!["string"], move |args| {
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::String(string) => Ok(val::int64(string.len().try_into().unwrap())),
                _ => Err(invalid_args("Std.String.length")),
            },
            _ => Err(invalid_args("Std.String.length")),
        }
    }));

    module
}
