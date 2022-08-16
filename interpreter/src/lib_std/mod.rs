use crate::{
    environment::{Environment, Imports},
    module::Module,
};
use rogato_common::{
    ast::{
        expression::FnDefArgs,
        fn_def::{FnDef, FnDefBody},
    },
    native_fn::{NativeFn, NativeFnError},
    val,
    val::{Value, ValueRef},
};
use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;

pub mod list;
pub mod math;
pub mod string;

pub fn env() -> Environment {
    let mut env = Environment::new();

    let std_mod = std_module();
    let math_mod = math::module();
    let string_mod = string::module();
    let list_mod = list::module();

    env.import(&math_mod, Imports::All);

    env.define_module(std_mod);
    env.define_module(math_mod);
    env.define_module(string_mod);
    env.define_module(list_mod);

    env
}

pub fn std_module() -> Module {
    let mut module = Module::new("Std");

    module.fn_def(op_fn_def("+", move |args| {
        with_number_op_args("+", args, |a, b| Ok(a.saturating_add(b)))
    }));

    module.fn_def(op_fn_def("-", move |args| {
        with_number_op_args("-", args, |a, b| Ok(a.saturating_sub(b)))
    }));

    module.fn_def(op_fn_def("*", move |args| {
        with_number_op_args("*", args, |a, b| Ok(a.saturating_mul(b)))
    }));

    module.fn_def(op_fn_def("/", move |args| {
        with_number_op_args("/", args, |a, b| {
            a.checked_div(b).map_or(Err(invalid_args("/")), Ok)
        })
    }));

    module.fn_def(op_fn_def("%", move |args| {
        with_number_op_args("%", args, |a, b| {
            a.checked_rem(b).map_or(Err(invalid_args("%")), Ok)
        })
    }));

    module.fn_def(op_fn_def("^", move |args| {
        with_number_op_args("^", args, |a, b| {
            a.checked_powd(b).map_or(Err(invalid_args("^")), Ok)
        })
    }));

    module.fn_def(op_fn_def("++", move |args| {
        with_string_op_args("++", args, |a, b| Ok(format!("{}{}", a, b)))
    }));

    module.fn_def(fn_def("inspect", vec!["value"], move |args| {
        match args.get(0) {
            Some(value) => Ok(val::string(format!("{}", value))),
            None => Err(invalid_args("inspect")),
        }
    }));

    module.fn_def(fn_def("==", vec!["a", "b"], move |args| {
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => Ok(val::bool(a.eq(b))),
            _ => Err(invalid_args("==")),
        }
    }));

    module.fn_def(fn_def("!=", vec!["a", "b"], move |args| {
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => Ok(val::bool(a.ne(b))),
            _ => Err(invalid_args("!=")),
        }
    }));

    module.fn_def(fn_def(
        "range",
        vec!["start", "finish"],
        move |args| match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
                (Value::Number(start), Value::Number(end)) => {
                    let mut numbers: Vec<ValueRef> = vec![];
                    let start: i64 = start.trunc().to_i64().unwrap();
                    let end = end.trunc().to_i64().unwrap();
                    for i in start..end {
                        numbers.push(val::number(i))
                    }
                    Ok(val::list(numbers))
                }
                _ => Err(invalid_args("range")),
            },
            _ => Err(invalid_args("range")),
        },
    ));

    module
}

pub fn fn_def(id: &str, args: Vec<&str>, body: NativeFn) -> Rc<FnDef> {
    FnDef::new(
        id.to_string(),
        FnDefArgs::new(args.iter().map(|a| a.into()).collect()),
        Rc::new(FnDefBody::native(body)),
    )
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
    func: fn(Decimal, Decimal) -> Result<Decimal, NativeFnError>,
) -> Result<ValueRef, NativeFnError> {
    match (args.len(), args.get(0), args.get(1)) {
        (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
            (Value::Number(a), Value::Number(b)) => func(*a, *b).map(val::number),
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
    match (args.len(), args.get(0), args.get(1)) {
        (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
            (Value::String(a), Value::String(b)) => func(a, b).map(val::string),
            _ => Err(invalid_args(id)),
        },
        _ => Err(invalid_args(id)),
    }
}

pub fn invalid_args(id: &str) -> NativeFnError {
    NativeFnError::InvalidArguments(id.into())
}

pub fn unknown_err<E: Debug>(id: &str, error: E) -> NativeFnError {
    NativeFnError::Unknown(id.into(), format!("{:?}", error))
}
