use crate::{ast::Identifier, val::ValueRef};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum NativeFnError {
    #[allow(dead_code)]
    #[error("NativeFnError: Unknown error: {0}")]
    Unknown(String),

    #[error("NativeFnError: Invalid arguments for {0}")]
    InvalidArguments(Identifier),
}

pub type NativeFn = fn(args: &[ValueRef]) -> Result<ValueRef, NativeFnError>;
