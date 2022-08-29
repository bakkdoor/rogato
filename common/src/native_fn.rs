use crate::{ast::Identifier, val::ValueRef};
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

pub type NativeFn = fn(args: &[ValueRef]) -> Result<ValueRef, NativeFnError>;
