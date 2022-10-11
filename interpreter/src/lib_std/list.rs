use super::{fn_def, invalid_args};
use crate::{module::Module, EvalContext};
use rogato_common::{
    native_fn::NativeFnError,
    val::{self, Value, ValueRef},
};
use std::{ops::Deref, rc::Rc};

pub fn module() -> Module {
    let mut module = Module::new("Std.List");

    module.fn_def(fn_def(
        "map",
        vec!["list", "f"],
        move |_ctx, args| -> Result<Rc<Value>, NativeFnError> {
            let error = Err(invalid_args("Std.List.map"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (a.deref(), b.deref()) {
                    (Value::List(items), Value::Symbol(fn_id)) => {
                        let mut result: Vec<ValueRef> = Vec::new();
                        let mut context = EvalContext::new();
                        for item in items.iter() {
                            match context.call_function(fn_id, &[Rc::clone(item)]) {
                                Some(val) => result.push(Rc::clone(&val?)),
                                None => {
                                    return Err(NativeFnError::EvaluationFailed(format!(
                                        "FunctionRef invalid: {}",
                                        fn_id
                                    )))
                                }
                            }
                        }
                        Ok(val::list(result))
                    }
                    (Value::List(items), Value::Lambda(lambda_ctx, lambda)) => {
                        let mut result: Vec<ValueRef> = Vec::new();
                        let mut context = EvalContext::new();
                        for item in items.iter() {
                            let val = context.call_lambda(
                                Rc::clone(lambda_ctx),
                                lambda,
                                &[Rc::clone(item)],
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
    ));

    module
}
