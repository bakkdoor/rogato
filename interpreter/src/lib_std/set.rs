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
        "from".into(),
        "merge".into(),
        "map".into(),
        "length".into(),
    ]));

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
        "merge",
        &["list1", "list2"],
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
        "map",
        &["list", "f"],
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
                                        format!("FunctionRef invalid in ^map: ^{}", fn_id),
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

    module.fn_def_native("length", &["list"], move |_ctx, args| {
        let error = Err(invalid_args("Std.Set.length"));

        match (args.len(), args.get(0)) {
            (1, Some(map1)) => match &**map1 {
                Value::Set(list) => Ok(val::number(list.len())),
                _ => error,
            },

            (_, _) => error,
        }
    });

    module
}
