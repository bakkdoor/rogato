use crate::{
    environment::{Environment, Imports},
    module::Module,
};
use rand::Rng;
use rogato_common::{
    ast::{
        expression::FnDefArgs,
        fn_def::{FnDefBody, FnDefVariant},
        module_def::ModuleExports,
    },
    native_fn::{NativeFn, NativeFnError},
    val::{self, List},
    val::{Value, ValueRef},
};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;

pub mod list;
pub mod map;
pub mod math;
pub mod string;
pub mod symbol;

pub fn env() -> Environment {
    let mut env = Environment::new();

    let std_mod = std_module();
    let math_mod = math::module();
    let string_mod = string::module();
    let list_mod = list::module();
    let map_mod = map::module();
    let symbol_mod = symbol::module();

    env.import(&std_mod, Imports::All);
    env.import(&math_mod, Imports::All);

    env.alias_module(&math_mod, "Math");
    env.alias_module(&string_mod, "String");
    env.alias_module(&list_mod, "List");
    env.alias_module(&map_mod, "Map");
    env.alias_module(&symbol_mod, "Symbol");

    env.define_module(std_mod);
    env.define_module(math_mod);
    env.define_module(string_mod);
    env.define_module(list_mod);
    env.define_module(map_mod);
    env.define_module(symbol_mod);

    env
}

pub fn std_module() -> Module {
    let mut module = Module::new("Std");
    module.export(&ModuleExports::new(vec![
        "++".into(),
        "id".into(),
        "print".into(),
        "println".into(),
        "apply".into(),
        "toString".into(),
        "inspect".into(),
        ">".into(),
        "<".into(),
        ">=".into(),
        "<=".into(),
        "==".into(),
        "!=".into(),
        "range".into(),
        "random".into(),
        "length".into(),
        "match".into(),
    ]));

    module.fn_def(
        "++",
        op_fn(move |_ctx, args| with_string_op_args("++", args, |a, b| Ok(format!("{}{}", a, b)))),
    );

    module.fn_def_native("id", &["value"], move |_ctx, args| match args.get(0) {
        Some(value) => Ok(ValueRef::clone(value)),
        None => Err(invalid_args("id")),
    });

    module.fn_def_native("print", &["value"], move |_ctx, args| match args.get(0) {
        Some(value) => {
            print!("{}", value);
            Ok(ValueRef::clone(value))
        }
        None => Err(invalid_args("print")),
    });

    module.fn_def_native("println", &["value"], move |_ctx, args| match args.get(0) {
        Some(value) => {
            println!("{}", value);
            Ok(ValueRef::clone(value))
        }
        None => Err(invalid_args("println")),
    });

    module.fn_def_native("apply", &["func", "?args"], move |ctx, args| {
        let error = Err(invalid_args("apply"));
        match (args.len(), args.get(0), args.get(1)) {
            (1, Some(func), None) => Ok(ValueRef::clone(func)),
            (2, Some(func), Some(args)) => match (func.deref(), args.deref()) {
                (Value::Lambda(lambda_ctx, lambda), Value::List(args)) => {
                    let args: Vec<ValueRef> = args.iter().map(ValueRef::clone).collect();
                    lambda_ctx
                        .borrow_mut()
                        .evaluate_lambda_call(lambda, &args)
                        .map_err(|e| {
                            NativeFnError::EvaluationFailed(func.to_string().into(), e.to_string())
                        })
                }
                (Value::Symbol(fn_id), Value::List(args)) => {
                    let args: Vec<ValueRef> = args.iter().map(ValueRef::clone).collect();
                    match ctx.call_function(fn_id, &args) {
                        Some(val) => Ok(ValueRef::clone(&val?)),
                        None => Err(NativeFnError::EvaluationFailed(
                            fn_id.clone(),
                            format!("FunctionRef invalid in ^apply: ^{}", fn_id),
                        )),
                    }
                }
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("toString", &["value"], move |_ctx, args| {
        match args.get(0) {
            Some(value) => match value.deref() {
                Value::String(_) => Ok(ValueRef::clone(value)),
                _ => Ok(val::string(format!("{}", value))),
            },
            None => Err(invalid_args("inspect")),
        }
    });

    module.fn_def_native("inspect", &["value"], move |_ctx, args| match args.get(0) {
        Some(value) => Ok(val::string(format!("{}", value))),
        None => Err(invalid_args("inspect")),
    });

    module.fn_def_native(">", &["a", "b"], move |_ctx, args| {
        let error = Err(invalid_args(">"));
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
                (Value::Number(a), Value::Number(b)) => Ok(val::bool(a.gt(b))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("<", &["a", "b"], move |_ctx, args| {
        let error = Err(invalid_args(">"));
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
                (Value::Number(a), Value::Number(b)) => Ok(val::bool(a.lt(b))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native(">=", &["a", "b"], move |_ctx, args| {
        let error = Err(invalid_args(">"));
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
                (Value::Number(a), Value::Number(b)) => Ok(val::bool(a.ge(b))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("<=", &["a", "b"], move |_ctx, args| {
        let error = Err(invalid_args(">"));
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
                (Value::Number(a), Value::Number(b)) => Ok(val::bool(a.le(b))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("==", &["a", "b"], move |_ctx, args| {
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => Ok(val::bool(a.eq(b))),
            _ => Err(invalid_args("==")),
        }
    });

    module.fn_def_native("!=", &["a", "b"], move |_ctx, args| {
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(a), Some(b)) => Ok(val::bool(a.ne(b))),
            _ => Err(invalid_args("!=")),
        }
    });

    module.fn_def_native("range", &["?start", "end"], move |_ctx, args| {
        let error = Err(invalid_args("range"));
        match (args.len(), args.get(0), args.get(1)) {
            (1, Some(a), None) => match (*a).deref() {
                Value::Number(end) => {
                    if *end < dec!(0) {
                        return error;
                    }
                    let start = 0i64;
                    let end = end.trunc().to_i64().unwrap();
                    let numbers = List::from_iter((start..end).map(val::number));
                    Ok(numbers.into())
                }
                _ => error,
            },
            (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
                (Value::Number(start), Value::Number(end)) => {
                    let start: i64 = start.trunc().to_i64().unwrap();
                    let end = end.trunc().to_i64().unwrap();
                    let numbers = List::from_iter((start..end).map(val::number));
                    Ok(numbers.into())
                }
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("random", &["min", "?max"], move |_ctx, args| {
        let error = Err(invalid_args("random"));
        match (args.len(), args.get(0), args.get(1)) {
            (1, Some(a), None) => match (*a).deref() {
                Value::Number(max) => {
                    if *max == dec!(0) {
                        return Ok(val::number(0));
                    }
                    let mut rng = rand::rngs::OsRng;
                    if *max < dec!(0) {
                        Ok(val::number(rng.gen_range(*max..dec!(0))))
                    } else {
                        Ok(val::number(rng.gen_range(dec!(0)..*max)))
                    }
                }
                _ => error,
            },
            (2, Some(a), Some(b)) => match ((*a).deref(), (*b).deref()) {
                (Value::Number(min), Value::Number(max)) => {
                    let mut rng = rand::rngs::OsRng;
                    if *min == *max {
                        return Ok(val::number(*min));
                    }
                    if *min < *max {
                        Ok(val::number(rng.gen_range(*min..*max)))
                    } else {
                        Ok(val::number(rng.gen_range(*max..*min)))
                    }
                }
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native(
        "length",
        &["collection"],
        move |_ctx, args| -> Result<Rc<Value>, NativeFnError> {
            let error = Err(invalid_args("length"));
            match (args.len(), args.get(0)) {
                (1, Some(l)) => match (*l).deref() {
                    Value::String(s) => Ok(val::number(s.len())),
                    Value::List(list) => Ok(val::number(list.len())),
                    Value::Map(map) => Ok(val::number(map.len())),
                    Value::Queue(queue) => Ok(val::number(queue.len())),
                    Value::Set(set) => Ok(val::number(set.len())),
                    Value::Stack(stack) => Ok(val::number(stack.len())),
                    Value::Vector(vector) => Ok(val::number(vector.len())),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "match",
        &["val", "fn"],
        move |ctx, args| -> Result<Rc<Value>, NativeFnError> {
            let error = Err(invalid_args("match"));

            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(val), Some(func)) => match &**func {
                    Value::Lambda(lambda_ctx, lambda) => lambda_ctx
                        .borrow_mut()
                        .evaluate_lambda_call(lambda.as_ref(), &[ValueRef::clone(val)])
                        .map_err(|e| e.into()),

                    Value::Symbol(fn_id) => {
                        match ctx.call_function(fn_id, &[ValueRef::clone(val)]) {
                            Some(val) => Ok(ValueRef::clone(&val?)),
                            None => Err(NativeFnError::EvaluationFailed(
                                fn_id.clone(),
                                format!("FunctionRef invalid in ^match: ^{}", fn_id),
                            )),
                        }
                    }

                    _ => error,
                },
                (_, _, _) => error,
            }
        },
    );

    module
}

pub fn op_fn(body: NativeFn) -> FnDefVariant {
    FnDefVariant(
        FnDefArgs::new(vec![Rc::new("left".into()), Rc::new("right".into())]),
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
