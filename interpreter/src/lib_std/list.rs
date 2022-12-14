use super::invalid_args;
use crate::module::Module;
use rogato_common::{
    ast::module_def::ModuleExports,
    native_fn::NativeFnError,
    val::{self, Value, ValueRef},
};
use std::rc::Rc;

pub fn module() -> Module {
    let mut module = Module::new("Std.List");
    module.export(&ModuleExports::new(vec![
        "join".into(),
        "map".into(),
        "reverse".into(),
        "head".into(),
        "tail".into(),
        "length".into(),
    ]));

    module.fn_def_native(
        "join",
        &["list1", "list2"],
        move |_ctx, args| -> Result<Rc<Value>, NativeFnError> {
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
        move |context, args| -> Result<Rc<Value>, NativeFnError> {
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
                                        format!("FunctionRef invalid in ^map: ^{}", fn_id),
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
        move |_ctx, args| -> Result<Rc<Value>, NativeFnError> {
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
        move |_ctx, args| -> Result<Rc<Value>, NativeFnError> {
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
        move |_ctx, args| -> Result<Rc<Value>, NativeFnError> {
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

    module
}
