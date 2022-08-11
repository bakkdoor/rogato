use std::ops::Deref;

use rogato_common::val::{self, Value};

use crate::module::Module;

use super::{fn_def, invalid_args};

pub fn module() -> Module {
    let mut module = Module::new("Std.Math");

    module.fn_def(fn_def("abs", vec!["num"], move |args| match args.get(0) {
        Some(val) => match val.deref() {
            Value::Int64(num) => Ok(val::int64(num.abs())),
            _ => Err(invalid_args("abs")),
        },
        _ => Err(invalid_args("abs")),
    }));

    module
}
