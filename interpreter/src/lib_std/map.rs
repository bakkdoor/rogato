use std::{ops::Deref, rc::Rc};

use crate::module::Module;
use rogato_common::val::{self, Value, ValueRef};

use super::invalid_args;

pub fn module() -> Module {
    let mut module = Module::new("Std.Map");

    module.fn_def_native("new", &[], move |_ctx, _args| {
        Ok(ValueRef::new(Value::Map(val::Map::new())))
    });

    module.fn_def_native("insert", &["map", "key", "?value"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Map.insert"));

        match (args.len(), args.get(0), args.get(1), args.get(2)) {
            (2, Some(map), Some(kv_pair), None) => match (map.deref(), kv_pair.deref()) {
                (Value::Map(map), Value::Tuple(2, pair)) => match (pair.get(0), pair.get(1)) {
                    (Some(key), Some(value)) => {
                        Ok(map.insert(Rc::clone(key), Rc::clone(value)).into())
                    }
                    _ => error,
                },
                _ => error,
            },

            (3, Some(map), Some(key), Some(value)) => match map.deref() {
                Value::Map(map) => Ok(map.insert(Rc::clone(key), Rc::clone(value)).into()),
                _ => error,
            },

            (_, _, _, _) => error,
        }
    });

    module.fn_def_native("remove", &["map", "key"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Map.remove"));

        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(map), Some(key)) => match map.deref() {
                Value::Map(map) => Ok(map.remove(key).into()),
                _ => error,
            },

            (_, _, _) => error,
        }
    });

    module.fn_def_native("merge", &["map1", "map2"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Map.merge"));

        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(map1), Some(map2)) => match (map1.deref(), map2.deref()) {
                (Value::Map(map1), Value::Map(map2)) => Ok(map1.merge(map2).into()),
                _ => error,
            },

            (_, _, _) => error,
        }
    });

    module
}
