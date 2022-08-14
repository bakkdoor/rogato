use super::{fn_def, invalid_args};
use crate::module::Module;
use rogato_common::val::{self, Value};
use std::ops::Deref;

pub fn module() -> Module {
    let mut module = Module::new("Std.Math");

    module.fn_def(fn_def("abs", vec!["num"], move |args| match args.get(0) {
        Some(val) => match val.deref() {
            Value::Decimal(num) => Ok(val::decimal(num.abs())),
            _ => Err(invalid_args("abs")),
        },
        _ => Err(invalid_args("abs")),
    }));

    module
}
