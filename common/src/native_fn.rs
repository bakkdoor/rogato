use crate::{
    ast::{fn_def::FnDef, lambda::LambdaClosureEvalError, Identifier},
    val::ValueRef,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum NativeFnError {
    #[error("Unknown error in NativeFn {0}: {1}")]
    Unknown(Identifier, String),

    #[error("Invalid arguments for NativeFn {0}")]
    InvalidArguments(Identifier),

    #[error("Evaluation failed for NativeFn with: {0}")]
    EvaluationFailed(String),
}

impl From<LambdaClosureEvalError> for NativeFnError {
    fn from(e: LambdaClosureEvalError) -> Self {
        NativeFnError::EvaluationFailed(e.to_string())
    }
}

pub trait NativeFnContext {
    fn lookup_var(&self, id: Identifier) -> Option<ValueRef>;
    fn lookup_const(&self, id: &Identifier) -> Option<ValueRef>;

    fn call_function(
        &mut self,
        id: &Identifier,
        args: &[ValueRef],
    ) -> Option<Result<ValueRef, NativeFnError>>;

    fn call_function_direct(
        &mut self,
        func: &FnDef,
        args: &[ValueRef],
    ) -> Result<ValueRef, NativeFnError>;
}

pub type NativeFn =
    fn(ctx: &mut dyn NativeFnContext, args: &[ValueRef]) -> Result<ValueRef, NativeFnError>;
