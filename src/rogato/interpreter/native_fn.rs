use std::rc::Rc;

use crate::rogato::{
    ast::{
        expression::FnDefArgs,
        fn_def::{FnDef, FnDefBody},
    },
    db::val,
    db::val::Value,
};

use super::{environment::Environment, EvalContext};

pub type NativeFn = fn(context: &mut EvalContext, args: &[Value]) -> Value;

pub fn std_env() -> Environment {
    let env = Environment::new_with_module("Std.Math");
    let mut module = env.current_module();
    let op_args = FnDefArgs::new(vec!["a".to_string(), "b".to_string()]);

    module.fn_def(FnDef::new(
        "+".to_string(),
        op_args.clone(),
        Rc::new(FnDefBody::native(move |_ctx, args| {
            with_number_op_args("+", args, |a, b| a + b)
        })),
    ));

    module.fn_def(FnDef::new(
        "-".to_string(),
        op_args.clone(),
        Rc::new(FnDefBody::native(move |_ctx, args| {
            with_number_op_args("-", args, |a, b| a - b)
        })),
    ));

    module.fn_def(FnDef::new(
        "*".to_string(),
        op_args.clone(),
        Rc::new(FnDefBody::native(move |_ctx, args| {
            with_number_op_args("*", args, |a, b| a * b)
        })),
    ));

    module.fn_def(FnDef::new(
        "/".to_string(),
        op_args,
        Rc::new(FnDefBody::native(move |_ctx, args| {
            with_number_op_args("/", args, |a, b| a / b)
        })),
    ));

    env
}

fn with_number_op_args(id: &str, args: &[Value], func: fn(i64, i64) -> i64) -> Value {
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    match (Value::is_i64(a), Value::is_i64(b)) {
        (true, true) => val::number(func(a.as_i64().unwrap(), b.as_i64().unwrap())),
        _ => panic!("Invalid arguments for {}", id),
    }
}
