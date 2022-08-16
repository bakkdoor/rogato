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
        move |args| -> Result<Rc<Value>, NativeFnError> {
            let error = Err(invalid_args("Std.List.map"));
            match (args.len(), args.get(0), args.get(1)) {
                (2, Some(a), Some(b)) => match (a.deref(), b.deref()) {
                    (Value::List(items), Value::FunctionRef(fn_def)) => {
                        //context.call_function(fn_def.id())
                        let mut result: Vec<ValueRef> = Vec::new();
                        let mut context = EvalContext::new();
                        for item in items.iter() {
                            let val =
                                context.call_function_direct(fn_def, &vec![Rc::clone(&item)])?;
                            result.push(val)
                        }
                        Ok(val::list(result))
                    }
                    (Value::List(items), Value::Lambda(lambda)) => {
                        let mut result: Vec<ValueRef> = Vec::new();
                        let mut context = EvalContext::new();
                        for item in items.iter() {
                            let val = context.call_lambda(lambda, &vec![Rc::clone(&item)])?;
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
