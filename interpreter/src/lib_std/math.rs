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

    module.fn_def(fn_def("trunc", vec!["num"], move |args| {
        let error = Err(invalid_args("trunc"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Decimal(num) => Ok(val::decimal(num.trunc())),
                _ => error,
            },
            _ => error,
        }
    }));

    module.fn_def(fn_def("fract", vec!["num"], move |args| {
        let error = Err(invalid_args("fract"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Decimal(num) => Ok(val::decimal(num.fract())),
                _ => error,
            },
            _ => error,
        }
    }));

    module.fn_def(fn_def("max", vec!["a", "b"], move |args| {
        let error = Err(invalid_args("max"));
        match (args.get(0), args.get(1)) {
            (Some(a), Some(b)) => match (a.deref(), b.deref()) {
                (Value::Decimal(ad), Value::Decimal(bd)) => Ok(val::decimal((*ad).max(*bd))),
                _ => error,
            },
            _ => error,
        }
    }));

    module.fn_def(fn_def("min", vec!["a", "b"], move |args| {
        let error = Err(invalid_args("min"));
        match (args.get(0), args.get(1)) {
            (Some(a), Some(b)) => match (a.deref(), b.deref()) {
                (Value::Decimal(ad), Value::Decimal(bd)) => Ok(val::decimal((*ad).min(*bd))),
                _ => error,
            },
            _ => error,
        }
    }));

    module
}
