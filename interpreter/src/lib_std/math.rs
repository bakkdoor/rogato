use super::invalid_args;
use crate::module::Module;
use rogato_common::val::{self, Value};
use rust_decimal::MathematicalOps;
use std::ops::Deref;

pub fn module() -> Module {
    let mut module = Module::new("Std.Math");

    module.fn_def_native("abs", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("abs"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Number(num) => Ok(val::number(num.abs())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("round", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("round"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Number(num) => Ok(val::number(num.round())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("ceil", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("ceil"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Number(num) => Ok(val::number(num.ceil())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("floor", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("floor"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Number(num) => Ok(val::number(num.floor())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("trunc", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("trunc"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Number(num) => Ok(val::number(num.trunc())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("fract", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("fract"));
        match args.get(0) {
            Some(val) => match val.deref() {
                Value::Number(num) => Ok(val::number(num.fract())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("max", &["a", "b"], move |_ctx, args| {
        let error = Err(invalid_args("max"));
        match (args.get(0), args.get(1)) {
            (Some(a), Some(b)) => match (a.deref(), b.deref()) {
                (Value::Number(ad), Value::Number(bd)) => Ok(val::number((*ad).max(*bd))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("min", &["a", "b"], move |_ctx, args| {
        let error = Err(invalid_args("min"));
        match (args.get(0), args.get(1)) {
            (Some(a), Some(b)) => match (a.deref(), b.deref()) {
                (Value::Number(ad), Value::Number(bd)) => Ok(val::number((*ad).min(*bd))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("sqrt", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("srqt"));
        match args.get(0) {
            Some(a) => match a.deref() {
                Value::Number(num) => Ok(val::option(num.sqrt().map(val::number))),
                _ => error,
            },
            _ => error,
        }
    });

    module
}
