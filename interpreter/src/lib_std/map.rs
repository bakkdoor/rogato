use std::rc::Rc;

use crate::module::Module;
use rogato_common::{
    ast::module_def::ModuleExports,
    val::{self, Value, ValueRef},
};

use super::invalid_args;

pub fn module() -> Module {
    let mut module = Module::new("Std.Map");
    module.export(&ModuleExports::new(vec![
        "new".into(),
        "contains".into(),
        "insert".into(),
        "insertOrUpdate".into(),
        "remove".into(),
        "merge".into(),
        "length".into(),
    ]));

    module.fn_def_native("new", &[], move |_ctx, _args| {
        Ok(ValueRef::new(Value::Map(val::Map::new())))
    });

    module.fn_def_native("contains", &["map", "key"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Map.contains"));

        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(map), Some(key)) => match &**map {
                Value::Map(map) => Ok(val::bool(map.contains(key))),
                _ => error,
            },
            (_, _, _) => error,
        }
    });

    module.fn_def_native("insert", &["map", "key", "?value"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Map.insert"));

        match (args.len(), args.get(0), args.get(1), args.get(2)) {
            (2, Some(map), Some(kv_pair), None) => match (&**map, &**kv_pair) {
                (Value::Map(map), Value::Tuple(2, pair)) => match (pair.get(0), pair.get(1)) {
                    (Some(key), Some(value)) => Ok(map
                        .insert(ValueRef::clone(key), ValueRef::clone(value))
                        .into()),
                    _ => error,
                },
                _ => error,
            },

            (3, Some(map), Some(key), Some(value)) => match &**map {
                Value::Map(map) => Ok(map
                    .insert(ValueRef::clone(key), ValueRef::clone(value))
                    .into()),
                _ => error,
            },

            (_, _, _, _) => error,
        }
    });

    module.fn_def_native(
        "insertOrUpdate",
        &["map", "key", "value", "func"],
        move |ctx, args| {
            let error = Err(invalid_args("Std.Map.insertOrUpdate"));

            match (
                args.len(),
                args.get(0),
                args.get(1),
                args.get(2),
                args.get(3),
            ) {
                (4, Some(map), Some(key), Some(value), Some(func)) => match (&**map, &**func) {
                    (Value::Map(map), Value::Lambda(lambda_ctx, lambda)) => match map.get(key) {
                        Some(value) => {
                            let value = ctx.call_lambda(Rc::clone(lambda_ctx), lambda, &[value])?;

                            Ok(map.insert(ValueRef::clone(key), value).into())
                        }
                        None => Ok(map
                            .insert(ValueRef::clone(key), ValueRef::clone(value))
                            .into()),
                    },
                    (Value::Map(map), Value::Symbol(fn_id)) => match map.get(key) {
                        Some(value) => match ctx.call_function(fn_id, &[value]) {
                            Some(Ok(value)) => Ok(map.insert(ValueRef::clone(key), value).into()),
                            Some(Err(e)) => Err(e),
                            None => Err(rogato_common::native_fn::NativeFnError::InvalidArguments(
                                "Std.Map.insertOrUpdate".into(),
                            )),
                        },
                        None => Ok(map
                            .insert(ValueRef::clone(key), ValueRef::clone(value))
                            .into()),
                    },
                    _ => error,
                },

                (_, _, _, _, _) => error,
            }
        },
    );

    module.fn_def_native("remove", &["map", "key"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Map.remove"));

        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(map), Some(key)) => match &**map {
                Value::Map(map) => Ok(map.remove(key).into()),
                _ => error,
            },

            (_, _, _) => error,
        }
    });

    module.fn_def_native("merge", &["map1", "map2"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Map.merge"));

        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(map1), Some(map2)) => match (&**map1, &**map2) {
                (Value::Map(map1), Value::Map(map2)) => Ok(map1.merge(map2).into()),
                _ => error,
            },

            (_, _, _) => error,
        }
    });

    module.fn_def_native("length", &["map"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Map.length"));

        match (args.len(), args.get(0)) {
            (1, Some(map1)) => match &**map1 {
                Value::Map(map) => Ok(val::number(map.len())),
                _ => error,
            },

            (_, _) => error,
        }
    });

    module.fn_def_native(
        "getOrElse",
        &["map", "key", "default"],
        move |_ctx, args| {
            let error = Err(invalid_args("Std.Map.getOrElse"));

            match (args.len(), args.get(0), args.get(1), args.get(2)) {
                (3, Some(map), Some(key), Some(default)) => match &**map {
                    Value::Map(map) => Ok(map.get(key).unwrap_or(ValueRef::clone(default))),
                    _ => error,
                },

                (_, _, _, _) => error,
            }
        },
    );

    module.fn_def_native("filter", &["map", "func"], move |ctx, args| {
        let error = Err(invalid_args("Std.Map.filter"));

        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(map), Some(func)) => match (&**map, &**func) {
                (Value::Map(map), Value::Lambda(lambda_ctx, lambda)) => {
                    let mut new_map = val::Map::new();

                    for (key, value) in map.iter() {
                        let result = ctx.call_lambda(
                            Rc::clone(lambda_ctx),
                            lambda,
                            &[ValueRef::clone(key), ValueRef::clone(value)],
                        )?;

                        if result.is_truthy() {
                            new_map = new_map.insert(ValueRef::clone(key), ValueRef::clone(value));
                        }
                    }

                    Ok(ValueRef::new(Value::Map(new_map)))
                }
                (Value::Map(map), Value::Symbol(fn_id)) => {
                    let mut new_map = val::Map::new();

                    for (key, value) in map.iter() {
                        match ctx.call_function(fn_id, &[ValueRef::clone(value)]) {
                            Some(Ok(result)) => {
                                if result.is_truthy() {
                                    new_map = new_map
                                        .insert(ValueRef::clone(key), ValueRef::clone(value));
                                }
                            }
                            Some(Err(e)) => return Err(e),
                            None => {
                                return Err(
                                    rogato_common::native_fn::NativeFnError::InvalidArguments(
                                        "Std.Map.filter".into(),
                                    ),
                                )
                            }
                        }
                    }

                    Ok(ValueRef::new(Value::Map(new_map)))
                }
                _ => error,
            },

            (_, _, _) => error,
        }
    });

    module
}
