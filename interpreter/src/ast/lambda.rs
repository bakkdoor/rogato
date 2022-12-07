use std::{cell::RefCell, fmt::Display, rc::Rc};

use rogato_common::{
    ast::{
        lambda::{Lambda, LambdaArgs},
        ASTDepth, VarIdentifier,
    },
    val::{self, ValueRef},
};

#[cfg(feature = "flame_it")]
use flamer::flame;

use crate::{EvalContext, EvalError, Evaluate};

impl<A: Display + ASTDepth + Evaluate<ValueRef>> Evaluate<ValueRef> for LambdaArgs<A> {
    #[cfg_attr(feature = "flame_it", flame("LambdaArgs::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut vec = Vec::with_capacity(self.len());
        for arg in self.iter() {
            vec.push(arg.evaluate(context)?)
        }
        Ok(val::list(vec))
    }
}

impl<A: Display + ASTDepth + Evaluate<ValueRef>> Evaluate<Vec<ValueRef>> for LambdaArgs<A> {
    #[cfg_attr(feature = "flame_it", flame("LambdaArgs<Vec>::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<Vec<ValueRef>, EvalError> {
        let mut vec = Vec::with_capacity(self.len());
        for arg in self.iter() {
            vec.push(arg.evaluate(context)?)
        }
        Ok(vec)
    }
}

impl Evaluate<ValueRef> for VarIdentifier {
    #[cfg_attr(feature = "flame_it", flame("VarIdentifier::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match context.lookup_var(self) {
            Some(val) => Ok(val),
            None => Err(EvalError::VarNotDefined(self.clone())),
        }
    }
}

impl Evaluate<ValueRef> for Rc<Lambda> {
    #[cfg_attr(feature = "flame_it", flame("Lambda::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        Ok(val::lambda(
            Rc::new(RefCell::new(context.clone())),
            Rc::clone(self),
        ))
    }
}
