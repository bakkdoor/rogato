use super::invalid_args;
use crate::module::Module;
use rogato_common::{
    ast::module_def::ModuleExports,
    native_fn::NativeFnError,
    val::{self, Value, ValueRef},
};
use std::{collections::HashMap, rc::Rc};

pub fn module() -> Module {
    let mut module = Module::new("Std.List");
    module.export(&ModuleExports::new(vec![
        "join".into(),
        "map".into(),
        "reverse".into(),
        "head".into(),
        "tail".into(),
        "length".into(),
        "inChunksOf".into(),
        "countByGroups".into(),
        "intersperse".into(),
        "pairWithNext".into(),
    ]));

    module.fn_def_native(
        "join",
        &["list1", "list2"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.reverse"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (&**a, &**b) {
                    (Value::List(items1), Value::List(items2)) => Ok(items1.join(items2).into()),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "map",
        &["list", "f"],
        move |context, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.map"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (&**a, &**b) {
                    (Value::List(items), Value::Symbol(fn_id)) => {
                        let mut result: Vec<ValueRef> = Vec::with_capacity(items.len());
                        for item in items.iter() {
                            match context.call_function(fn_id, &[ValueRef::clone(item)]) {
                                Some(val) => result.push(ValueRef::clone(&val?)),
                                None => {
                                    return Err(NativeFnError::EvaluationFailed(
                                        fn_id.clone(),
                                        format!("FunctionRef invalid in ^map: ^{fn_id}"),
                                    ))
                                }
                            }
                        }
                        Ok(val::list(result))
                    }
                    (Value::List(items), Value::Lambda(lambda_ctx, lambda)) => {
                        let mut result: Vec<ValueRef> = Vec::with_capacity(items.len());
                        for item in items.iter() {
                            let val = context.call_lambda(
                                Rc::clone(lambda_ctx),
                                lambda,
                                &[ValueRef::clone(item)],
                            )?;
                            result.push(val)
                        }
                        Ok(val::list(result))
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "reverse",
        &["list"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.reverse"));
            match (args.len(), args.get(0)) {
                (1, Some(a)) => match &**a {
                    Value::List(items) => Ok(items.reverse().into()),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "head",
        &["list"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.head"));
            match (args.len(), args.get(0)) {
                (1, Some(a)) => match &**a {
                    Value::List(items) => {
                        if items.is_empty() {
                            error
                        } else {
                            Ok(items.head().unwrap())
                        }
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "tail",
        &["list"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.tail"));
            match (args.len(), args.get(0)) {
                (1, Some(a)) => match &**a {
                    Value::List(items) => Ok(items.tail().into()),
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native("length", &["list"], move |_ctx, args| {
        let error = Err(invalid_args("Std.List.length"));

        match (args.len(), args.get(0)) {
            (1, Some(map1)) => match &**map1 {
                Value::List(list) => Ok(val::number(list.len())),
                _ => error,
            },

            (_, _) => error,
        }
    });

    module.fn_def_native(
        "inChunksOf",
        &["list", "chunkSize"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.inChunksOf"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (&**a, &**b) {
                    (Value::List(items), Value::Number(chunk_size)) => {
                        if chunk_size.is_zero() {
                            return Err(NativeFnError::EvaluationFailed(
                                "Std.List.inChunksOf".into(),
                                "chunkSize must be greater than 0".into(),
                            ));
                        }
                        let chunk_size: usize = usize::try_from(*chunk_size).unwrap();
                        let mut result: Vec<ValueRef> =
                            Vec::with_capacity(items.len() / chunk_size);
                        for chunk in items.chunks(chunk_size) {
                            result.push(chunk.into())
                        }
                        Ok(val::list(result))
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "countByGroups",
        &["list", "groupByFn"],
        move |ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.countByGroups"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (&**a, &**b) {
                    (Value::List(items), Value::Symbol(fn_id)) => {
                        let mut result: HashMap<ValueRef, usize> = HashMap::new();
                        for item in items.iter() {
                            let key = match ctx.call_function(fn_id, &[ValueRef::clone(item)]) {
                                Some(val) => val?,
                                None => {
                                    return Err(NativeFnError::EvaluationFailed(
                                        fn_id.clone(),
                                        format!("FunctionRef invalid in ^countByGroups: ^{fn_id}"),
                                    ))
                                }
                            };
                            let count = result.entry(key).or_insert(0);
                            *count += 1;
                        }

                        Ok(val::map(
                            result
                                .iter()
                                .map(|(k, v)| (ValueRef::clone(k), val::number(*v))),
                        ))
                    }
                    (Value::List(items), Value::Lambda(lambda_ctx, lambda)) => {
                        let mut result: HashMap<ValueRef, usize> = HashMap::new();
                        for item in items.iter() {
                            let key = ctx.call_lambda(
                                Rc::clone(lambda_ctx),
                                lambda,
                                &[ValueRef::clone(item)],
                            )?;
                            let count = result.entry(key).or_insert(0);
                            *count += 1;
                        }

                        Ok(val::map(
                            result
                                .iter()
                                .map(|(k, v)| (ValueRef::clone(k), val::number(*v))),
                        ))
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "intersperse",
        &["list", "value"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.intersperse"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(value)) => match &**a {
                    Value::List(items) => {
                        let mut result: Vec<ValueRef> = Vec::with_capacity(items.len() * 2);
                        for item in items.iter() {
                            result.push(ValueRef::clone(item));
                            result.push(ValueRef::clone(value));
                        }
                        result.pop();
                        Ok(val::list(result))
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "pairWithNext",
        &["list"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.pairWithNext"));
            match (args.len(), args.get(0)) {
                (1, Some(a)) => match &**a {
                    Value::List(items) => {
                        let mut result: Vec<ValueRef> = Vec::with_capacity(items.len() * 2);
                        let mut iter = items.iter();
                        if let Some(first) = iter.next() {
                            let mut prev = first;
                            for item in iter {
                                result.push(val::tuple([
                                    ValueRef::clone(prev),
                                    ValueRef::clone(item),
                                ]));

                                prev = item;
                            }
                        }
                        Ok(val::list(result))
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module.fn_def_native(
        "pairWithPrevious",
        &["list"],
        move |_ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.pairWithPrevious"));
            match (args.len(), args.get(0)) {
                (1, Some(a)) => match &**a {
                    Value::List(items) => {
                        let mut result: Vec<ValueRef> = Vec::with_capacity(items.len() * 2);
                        let iter = items.iter();
                        let mut prev = &val::none();
                        for item in iter {
                            result.push(val::tuple([ValueRef::clone(prev), ValueRef::clone(item)]));
                            prev = item;
                        }

                        Ok(val::list(result))
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    // a list of common Std.List helper function used in many algorithms:

    module.fn_def_native(
        "reduceRight",
        &["list", "initial", "fn"],
        move |ctx, args| -> Result<ValueRef, NativeFnError> {
            let error = Err(invalid_args("Std.List.reduceRight"));
            match (args.len(), args.get(0), args.get(1), args.get(2)) {
                (3, Some(a), Some(initial), Some(fn_val)) => match (&**a, &**fn_val) {
                    (Value::List(items), Value::Symbol(fn_id)) => {
                        let mut result = ValueRef::clone(initial);
                        for item in items.reverse().iter() {
                            result = match ctx
                                .call_function(fn_id, &[result, ValueRef::clone(item)])
                            {
                                Some(val) => ValueRef::clone(&val?),
                                None => return Err(NativeFnError::EvaluationFailed(
                                    fn_id.clone(),
                                    format!(
                                        "FunctionRef invalid in ^Std.List.reduceRight: ^{fn_id}"
                                    ),
                                )),
                            }
                        }
                        Ok(result)
                    }
                    (Value::List(items), Value::Lambda(lambda_ctx, lambda)) => {
                        let mut result = ValueRef::clone(initial);
                        for item in items.reverse().iter() {
                            result = ctx.call_lambda(
                                Rc::clone(lambda_ctx),
                                lambda,
                                &[result, ValueRef::clone(item)],
                            )?;
                        }
                        Ok(result)
                    }
                    _ => error,
                },
                _ => error,
            }
        },
    );

    module
}
