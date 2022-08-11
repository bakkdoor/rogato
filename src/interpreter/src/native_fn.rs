use std::{ops::Deref, rc::Rc};

use super::environment::Environment;
use rogato_common::native_fn::{NativeFn, NativeFnError};
use rogato_common::val;
use rogato_common::{
    ast::{
        expression::FnDefArgs,
        fn_def::{FnDef, FnDefBody},
    },
    val::{Value, ValueRef},
};

pub fn std_env() -> Environment {
    let env = Environment::new_with_module("Std.Math");
    let mut module = env.current_module();

    module.fn_def(op_fn_def(
        "+",
        fn_body(move |args| with_number_op_args("+", args, |a, b| Ok(a + b))),
    ));

    module.fn_def(op_fn_def(
        "-",
        fn_body(move |args| with_number_op_args("-", args, |a, b| Ok(a - b))),
    ));

    module.fn_def(op_fn_def(
        "*",
        fn_body(move |args| with_number_op_args("*", args, |a, b| Ok(a * b))),
    ));

    module.fn_def(op_fn_def(
        "/",
        fn_body(move |args| with_number_op_args("/", args, |a, b| Ok(a / b))),
    ));

    module.fn_def(op_fn_def(
        "%",
        fn_body(move |args| with_number_op_args("%", args, |a, b| Ok(a % b))),
    ));

    module.fn_def(op_fn_def(
        "^",
        fn_body(move |args| {
            with_number_op_args("^", args, |a, b| match u32::try_from(b) {
                Ok(exponent) => Ok(a.pow(exponent)),
                Err(_) => Err(invalid_args("^")),
            })
        }),
    ));

    env
}

fn op_fn_def(id: &str, body: Rc<FnDefBody>) -> Rc<FnDef> {
    FnDef::new(
        id.to_string(),
        FnDefArgs::new(vec!["left".into(), "right".into()]),
        body,
    )
}

fn fn_body(f: NativeFn) -> Rc<FnDefBody> {
    Rc::new(FnDefBody::native(f))
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
