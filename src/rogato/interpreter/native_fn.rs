use std::rc::Rc;
use thiserror::Error;

use crate::rogato::{
    ast::{
        expression::FnDefArgs,
        fn_def::{FnDef, FnDefBody},
    },
    db::val,
    db::val::Value,
};

use super::{environment::Environment, EvalContext, EvalError, Identifier};

pub type NativeFn = fn(context: &mut EvalContext, args: &[Value]) -> Result<Value, EvalError>;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum NativeFnError {
    #[allow(dead_code)]
    #[error("NativeFnError: Unknown error: {0}")]
    Unknown(String),

    #[error("NativeFnError: Invalid arguments for {0}")]
    InvalidArguments(Identifier),
}

pub fn std_env() -> Environment {
    let env = Environment::new_with_module("Std.Math");
    let mut module = env.current_module();

    module.fn_def(op_fn_def(
        "+",
        fn_body(move |_ctx, args| with_number_op_args("+", args, |a, b| a + b)),
    ));

    module.fn_def(op_fn_def(
        "-",
        fn_body(move |_ctx, args| with_number_op_args("-", args, |a, b| a - b)),
    ));

    module.fn_def(op_fn_def(
        "*",
        fn_body(move |_ctx, args| with_number_op_args("*", args, |a, b| a * b)),
    ));

    module.fn_def(op_fn_def(
        "/",
        fn_body(move |_ctx, args| with_number_op_args("/", args, |a, b| a / b)),
    ));

    module.fn_def(op_fn_def(
        "%",
        fn_body(move |_ctx, args| with_number_op_args("%", args, |a, b| a % b)),
    ));

    env
}

fn op_fn_def(id: &str, body: Rc<FnDefBody>) -> Rc<FnDef> {
    FnDef::new(
        id.to_string(),
        FnDefArgs::new(vec!["left".to_string(), "right".to_string()]),
        body,
    )
}

fn fn_body(f: NativeFn) -> Rc<FnDefBody> {
    Rc::new(FnDefBody::native(f))
}

fn with_number_op_args(
    id: &str,
    args: &[Value],
    func: fn(i64, i64) -> i64,
) -> Result<Value, EvalError> {
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    match (Value::is_i64(a), Value::is_i64(b)) {
        (true, true) => Ok(val::number(func(a.as_i64().unwrap(), b.as_i64().unwrap()))),
        _ => Err(EvalError::NativeFnFailed(NativeFnError::InvalidArguments(
            id.to_string(),
        ))),
    }
}
