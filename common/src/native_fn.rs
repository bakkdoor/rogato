use crate::{ast::Identifier, val::ValueRef};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum NativeFnError {
    #[error("NativeFnError: Unknown error for {0}: {1}")]
    Unknown(Identifier, String),

    #[error("NativeFnError: Invalid arguments for {0}")]
    InvalidArguments(Identifier),
}

pub type NativeFn = fn(args: &[ValueRef]) -> Result<ValueRef, NativeFnError>;
