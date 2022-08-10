use std::fmt::Display;

use rogato_common::{
    ast::{
        lambda::{Lambda, LambdaArgs},
        ASTDepth,
    },
    val::{self, ValueRef},
};

use crate::eval::{EvalContext, EvalError, Evaluate};

impl Evaluate<ValueRef> for Lambda {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let evaluated_args: Vec<ValueRef> = self.args().evaluate(context)?;
        let mut fn_context = context.with_child_env();
        let given_args = evaluated_args.len();
        let expected_args = self.arg_count();

        if given_args != expected_args {
            return Err(EvalError::LambdaArityMismatch(expected_args, given_args));
        }

        for (i, arg_val) in evaluated_args.iter().enumerate().take(self.arg_count()) {
            let arg_name = self.get_arg(i).unwrap().clone();
            fn_context.define_var(&arg_name, arg_val.clone())
        }
        self.body().evaluate(&mut fn_context)
    }
}

impl<A: Display + ASTDepth + Evaluate<ValueRef>> Evaluate<ValueRef> for LambdaArgs<A> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut vec = Vec::new();
        for arg in self.iter() {
            vec.push(arg.evaluate(context)?)
        }
        Ok(val::list(vec))
    }
}

impl<A: Display + ASTDepth + Evaluate<ValueRef>> Evaluate<Vec<ValueRef>> for LambdaArgs<A> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<Vec<ValueRef>, EvalError> {
        let mut vec = Vec::new();
        for arg in self.iter() {
            vec.push(arg.evaluate(context)?)
        }
        Ok(vec)
    }
}
