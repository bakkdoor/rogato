use super::invalid_args;
use crate::module::Module;
use rogato_common::{
    ast::module_def::ModuleExports,
    val::{self, Value, ValueRef},
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

    module.fn_def_native("reverse", &["string"], move |_ctx, args| {
        let error = Err(invalid_args("Std.String.reverse"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::String(string) => Ok(val::string(string.chars().rev().collect::<String>())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("split", &["string", "pattern"], move |_ctx, args| {
        let error = Err(invalid_args("Std.String.split"));
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(string), Some(pattern)) => match (&**string, &**pattern) {
                (Value::String(string), Value::String(split_str)) => Ok(val::list(
                    string
                        .split(split_str)
                        .map(val::string)
                        .collect::<Vec<ValueRef>>(),
                )),
                _ => error,
            },
            _ => error,
        }
    });

    module
}
