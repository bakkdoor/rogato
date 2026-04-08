use std::rc::Rc;

use rogato_common::ast::{type_expression::TypeExpression, Identifier, VarIdentifier};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TypeCheckError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(VarIdentifier),

    #[error("Undefined function: {0}")]
    UndefinedFunction(Identifier),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: Rc<TypeExpression>,
        actual: Rc<TypeExpression>,
    },

    #[error("Argument count mismatch: expected {expected} arguments, got {actual}")]
    ArgumentCountMismatch { expected: usize, actual: usize },

    #[error("Type error at argument {position}: {message}")]
    ArgumentError { position: usize, message: String },

    #[error("Cannot infer type for expression")]
    CannotInferType,

    #[error("Unknown type checking error: {0}")]
    Unknown(String),
}
