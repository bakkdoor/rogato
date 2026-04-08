use std::rc::Rc;

use rogato_common::ast::{type_expression::TypeExpression, Identifier, VarIdentifier};
use rogato_common::span::Span;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TypeCheckError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(VarIdentifier, Option<Span>),

    #[error("Undefined function: {0}")]
    UndefinedFunction(Identifier, Option<Span>),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: Rc<TypeExpression>,
        actual: Rc<TypeExpression>,
        span: Option<Span>,
    },

    #[error("Argument count mismatch: expected {expected} arguments, got {actual}")]
    ArgumentCountMismatch {
        expected: usize,
        actual: usize,
        span: Option<Span>,
    },

    #[error("Type error at argument {position}: {message}")]
    ArgumentError {
        position: usize,
        message: String,
        span: Option<Span>,
    },

    #[error("Cannot infer type for expression")]
    CannotInferType(Option<Span>),

    #[error("Unknown type checking error: {0}")]
    Unknown(String),
}

impl TypeCheckError {
    /// Returns the source span associated with this error, if any.
    pub fn span(&self) -> Option<Span> {
        match self {
            TypeCheckError::UndefinedVariable(_, span) => *span,
            TypeCheckError::UndefinedFunction(_, span) => *span,
            TypeCheckError::TypeMismatch { span, .. } => *span,
            TypeCheckError::ArgumentCountMismatch { span, .. } => *span,
            TypeCheckError::ArgumentError { span, .. } => *span,
            TypeCheckError::CannotInferType(span) => *span,
            TypeCheckError::Unknown(_) => None,
        }
    }
}
