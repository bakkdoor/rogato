use crate::{environment::Environment, module::Module};
use rogato_common::{
    ast::{
        expression::FnDefArgs,
        fn_def::{FnDef, FnDefBody},
    },
    native_fn::{NativeFn, NativeFnError},
    val,
    val::{Value, ValueRef},
};
use std::ops::Deref;
use std::rc::Rc;
pub mod math;
pub mod string;

pub fn env() -> Environment {
    let mut env = Environment::new();

    let math_mod = math::module();
    let string_mod = string::module();
    env.define_module(std_module());
    env.define_module(math_mod);
    env.define_module(string_mod);

    env
}

pub fn std_module() -> Module {
    let mut module = Module::new("Std");

    module.fn_def(op_fn_def("+", move |args| {
        with_number_op_args("+", args, |a, b| Ok(a + b))
    }));

    module.fn_def(op_fn_def("-", move |args| {
        with_number_op_args("-", args, |a, b| Ok(a - b))
    }));

    module.fn_def(op_fn_def("*", move |args| {
        with_number_op_args("*", args, |a, b| Ok(a * b))
    }));

    module.fn_def(op_fn_def("/", move |args| {
        with_number_op_args("/", args, |a, b| Ok(a / b))
    }));

    module.fn_def(op_fn_def("%", move |args| {
        with_number_op_args("%", args, |a, b| Ok(a % b))
    }));

    module.fn_def(op_fn_def("^", move |args| {
        with_number_op_args("^", args, |a, b| match u32::try_from(b) {
            Ok(exponent) => Ok(a.pow(exponent)),
            Err(_) => Err(invalid_args("^")),
        })
    }));

    module.fn_def(op_fn_def("++", move |args| {
        with_string_op_args("++", args, |a, b| Ok(format!("{}{}", a, b)))
    }));

    module
}

pub fn op_fn_def(id: &str, body: NativeFn) -> Rc<FnDef> {
    FnDef::new(
        id.to_string(),
        FnDefArgs::new(vec!["left".into(), "right".into()]),
        Rc::new(FnDefBody::native(body)),
    )
}

pub fn with_number_op_args(
    id: &str,
    args: &[ValueRef],
    func: fn(i64, i64) -> Result<i64, NativeFnError>,
) -> Result<ValueRef, NativeFnError> {
    match (args.get(0), args.get(1)) {
        (Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
            (Value::Int64(a), Value::Int64(b)) => func(*a, *b).map(val::int64),
            _ => Err(invalid_args(id)),
        },
        _ => Err(invalid_args(id)),
    }
}

fn with_string_op_args(
    id: &str,
    args: &[ValueRef],
    func: fn(&String, &String) -> Result<String, NativeFnError>,
) -> Result<ValueRef, NativeFnError> {
    match (args.get(0), args.get(1)) {
        (Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
            (Value::String(a), Value::String(b)) => func(a, b).map(val::string),
            _ => Err(invalid_args(id)),
        },
        _ => Err(invalid_args(id)),
    }
}

pub fn invalid_args(id: &str) -> NativeFnError {
    NativeFnError::InvalidArguments(id.into())
}
