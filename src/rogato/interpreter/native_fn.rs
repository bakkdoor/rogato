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

pub type NativeFn = fn(context: &mut EvalContext, args: &Vec<Value>) -> Value;

pub fn std_env() -> Environment {
    let env = Environment::new_with_module("Std.Math");

    let f_add = FnDef::new(
        "+".to_string(),
        FnDefArgs::new(vec!["a".to_string(), "b".to_string()]),
        Rc::new(FnDefBody::native(move |_ctx, args| {
            let a = args.get(0).unwrap();
            let b = args.get(1).unwrap();
            match (Value::is_i64(a), Value::is_i64(b)) {
                (true, true) => val::number(a.as_i64().unwrap() + b.as_i64().unwrap()),
                _ => panic!("Invalid arguments for +"),
            }
        })),
    );

    env.current_module().fn_def(Rc::new(f_add));
    env
}
