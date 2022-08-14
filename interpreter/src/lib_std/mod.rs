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
use rust_decimal::{Decimal, MathematicalOps};
use std::ops::Deref;
use std::rc::Rc;
pub mod math;
pub mod string;

pub fn env() -> Environment {
    let mut env = Environment::new();

    let std_mod = std_module();
    let math_mod = math::module();
    let string_mod = string::module();

    env.import(&math_mod, Imports::All);

    env.define_module(std_mod);
    env.define_module(math_mod);
    env.define_module(string_mod);

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
    match (args.get(0), args.get(1)) {
        (Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
            (Value::Decimal(a), Value::Decimal(b)) => func(*a, *b).map(val::decimal),
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
