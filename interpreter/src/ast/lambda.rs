use std::{fmt::Display, rc::Rc};

use rogato_common::{
    ast::{
        lambda::{Lambda, LambdaArgs},
        ASTDepth,
    },
    val::{self, ValueRef},
};

use crate::{EvalContext, EvalError, Evaluate, Identifier};

impl Evaluate<ValueRef> for Rc<Lambda> {
    fn evaluate(&self, _context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        Ok(val::lambda(Rc::clone(self)))
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

impl Evaluate<ValueRef> for Identifier {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match context.lookup_var(self) {
            Some(val) => Ok(val),
            None => Err(EvalError::VarNotDefined(self.clone())),
        }
    }
}
