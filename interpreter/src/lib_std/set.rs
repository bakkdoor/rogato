use super::invalid_args;
use crate::module::Module;
use rogato_common::{
    ast::module_def::ModuleExports,
    native_fn::NativeFnError,
    val::{self, Value, ValueRef},
};
use std::rc::Rc;

pub fn module() -> Module {
    let mut module = Module::new("Std.Set");
    module.export(&ModuleExports::new(vec![
        "contains".into(),
        "empty".into(),
        "filter".into(),
        "from".into(),
        "insert".into(),
        "isDisjoint".into(),
        "isEmpty".into(),
        "isSubset".into(),
        "isSuperset".into(),
        "length".into(),
        "map".into(),
        "merge".into(),
        "remove".into(),
        "toList".into(),
    ]));

    module.fn_def_native(
        "contains",
        &["set", "value"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.contains"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(set), Some(value)) => match &**set {
                    Value::Set(set) => Ok(val::bool(set.contains(value))),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "empty",
        &[],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.empty"));
            match args.len() {
                0 => Ok(val::set([])),
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "filter",
        &["set", "f"],
        move |context, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.filter"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (&**a, &**b) {
                    (Value::Set(set), Value::Symbol(fn_id)) => {
                        let mut filtered_set = set.clone();
                        for item in set.iter() {
                            match context.call_function(fn_id, &[ValueRef::clone(item)]) {
                                Some(Ok(value)) => {
                                    if let Value::Bool(false) = &*value {
                                        filtered_set = filtered_set.remove(item);
                                    }
                                }
                                Some(Err(error)) => {
                                    return Err(NativeFnError::Unknown(
                                        fn_id.clone(),
                                        error.to_string(),
                                    ))
                                }
                                None => {
                                    return Err(NativeFnError::EvaluationFailed(
                                        fn_id.clone(),
                                        format!(
                                            "FunctionRef invalid in ^Std.List.filter: ^{}",
                                            fn_id
                                        ),
                                    ))
                                }
                            }
                        }
                        Ok(filtered_set.into())
                    }
                    (Value::Set(set), Value::Lambda(lambda_ctx, lambda)) => {
                        let mut filtered_set = set.clone();
                        for item in set.iter() {
                            let val = context.call_lambda(
                                Rc::clone(lambda_ctx),
                                lambda,
                                &[ValueRef::clone(item)],
                            )?;
                            if let Value::Bool(false) = &*val {
                                filtered_set = filtered_set.remove(item);
                            }
                        }
                        Ok(filtered_set.into())
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "from",
        &["items"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.from"));
            match (args.len(), args.get(0)) {
                (1, Some(a)) => match &**a {
                    Value::List(items) => Ok(val::Set::from(items).into()),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "insert",
        &["set", "value"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.insert"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(set), Some(value)) => match &**set {
                    Value::Set(set) => Ok(set.insert(ValueRef::clone(value)).into()),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "isDisjoint",
        &["set1", "set2"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.isDisjoint"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(set1), Some(set2)) => match (&**set1, &**set2) {
                    (Value::Set(set1), Value::Set(set2)) => Ok(val::bool(set1.is_disjoint(set2))),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "isEmpty",
        &["set"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.isEmpty"));
            match (args.len(), args.get(0)) {
                (1, Some(set)) => match &**set {
                    Value::Set(set) => Ok(val::bool(set.is_empty())),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "isSubset",
        &["set1", "set2"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.isSubset"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(set1), Some(set2)) => match (&**set1, &**set2) {
                    (Value::Set(set1), Value::Set(set2)) => Ok(val::bool(set1.is_subset(set2))),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "isSuperset",
        &["set1", "set2"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.isSuperset"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(set1), Some(set2)) => match (&**set1, &**set2) {
                    (Value::Set(set1), Value::Set(set2)) => Ok(val::bool(set1.is_superset(set2))),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native("length", &["set"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Set.length"));

        match (args.len(), args.get(0)) {
            (1, Some(map1)) => match &**map1 {
                Value::Set(list) => Ok(val::number(list.len())),
                _ => error,
            },

            (_, _) => error,
        }
    });

    module.fn_def_native(
        "map",
        &["set", "f"],
        move |context, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.map"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (&**a, &**b) {
                    (Value::Set(items), Value::Symbol(fn_id)) => {
                        let mut result: Vec<ValueRef> = Vec::with_capacity(items.len());
                        for item in items.iter() {
                            match context.call_function(fn_id, &[ValueRef::clone(item)]) {
                                Some(val) => result.push(ValueRef::clone(&val?)),
                                None => {
                                    return Err(NativeFnError::EvaluationFailed(
                                        fn_id.clone(),
                                        format!("FunctionRef invalid in ^Std.List.map: ^{}", fn_id),
                                    ))
                                }
                            }
                        }
                        Ok(val::set(result))
                    }
                    (Value::Set(items), Value::Lambda(lambda_ctx, lambda)) => {
                        let mut result: Vec<ValueRef> = Vec::with_capacity(items.len());
                        for item in items.iter() {
                            let val = context.call_lambda(
                                Rc::clone(lambda_ctx),
                                lambda,
                                &[ValueRef::clone(item)],
                            )?;
                            result.push(val)
                        }
                        Ok(val::set(result))
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "merge",
        &["set1", "set2"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.reverse"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (&**a, &**b) {
                    (Value::Set(items1), Value::Set(items2)) => Ok(items1.merge(items2).into()),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "remove",
        &["set", "value"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.remove"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(set), Some(value)) => match &**set {
                    Value::Set(set) => Ok(set.remove(value).into()),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "toList",
        &["set"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.Set.toList"));
            match (args.len(), args.get(0)) {
                (1, Some(set)) => match &**set {
                    Value::Set(set) => Ok(set.to_list().into()),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module
}
