use super::{fn_def, invalid_args};
use crate::module::Module;
use rogato_common::val::{self, Value};
use std::ops::Deref;

pub fn module() -> Module {
    let mut module = Module::new("Std.Math");

    module.fn_def(fn_def("abs", vec!["num"], move |args| {
        let error = Err(invalid_args("abs"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Decimal(num) => Ok(val::decimal(num.abs())),
                _ => error,
            },
            _ => error,
        }
    }));

    module.fn_def(fn_def("round", vec!["num"], move |args| {
        let error = Err(invalid_args("round"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Decimal(num) => Ok(val::decimal(num.round())),
                _ => error,
            },
            _ => error,
        }
    }));

    module.fn_def(fn_def("ceil", vec!["num"], move |args| {
        let error = Err(invalid_args("ceil"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Decimal(num) => Ok(val::decimal(num.ceil())),
                _ => error,
            },
            _ => error,
        }
    }));

    module.fn_def(fn_def("floor", vec!["num"], move |args| {
        let error = Err(invalid_args("floor"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Decimal(num) => Ok(val::decimal(num.floor())),
                _ => error,
            },
            _ => error,
        }
    }));

    module
}
