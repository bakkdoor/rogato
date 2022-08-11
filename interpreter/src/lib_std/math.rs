use crate::module::Module;
use rogato_common::native_fn::{NativeFn, NativeFnError};
use rogato_common::val;
use rogato_common::{
    ast::{
        expression::FnDefArgs,
        fn_def::{FnDef, FnDefBody},
    },
    val::{Value, ValueRef},
};
use std::ops::Deref;
use std::rc::Rc;

pub fn module() -> Module {
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

    module
}

fn op_fn_def(id: &str, body: NativeFn) -> Rc<FnDef> {
    FnDef::new(
        id.to_string(),
        FnDefArgs::new(vec!["left".into(), "right".into()]),
        Rc::new(FnDefBody::native(body)),
    )
}

fn with_number_op_args(
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

fn invalid_args(id: &str) -> NativeFnError {
    NativeFnError::InvalidArguments(id.into())
}
