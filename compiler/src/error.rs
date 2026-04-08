use rogato_common::ast::Identifier;
use rogato_common::span::Span;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CodegenError {
    #[error("Unknown compiler codegen error: {0}")]
    Unknown(String),

    #[error("Compiler feature not yet implemented: {0}")]
    NotYetImplemented(String),

    #[error("Could not find a matching variable with name: {0}")]
    VarNotFound(Identifier, Option<Span>),

    #[error("Root comment ignored: {0}")]
    IgnoredRootComment(String),

    #[error("Function not defined: {0}")]
    FnNotDefined(Identifier, Option<Span>),

    #[error("Operator not defined: {0}")]
    OpNotDefined(Identifier, Option<Span>),

    #[error("FnDef codegen validation failed for: {0}")]
    FnDefValidationFailed(Identifier, Option<Span>),

    #[error("Type {0} not yet supported in codegen")]
    UnsupportedLLVMType(String),

    #[error("Cannot convert type {0} to {1}")]
    TypeConversionError(String, String),

    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    #[error("Function '{0}' has uncovered pattern - no catch-all in last variant")]
    FnPatternUncovered(Identifier, Option<Span>),
}

impl CodegenError {
    /// Returns the source span associated with this error, if any.
    pub fn span(&self) -> Option<Span> {
        match self {
            CodegenError::VarNotFound(_, span) => *span,
            CodegenError::FnNotDefined(_, span) => *span,
            CodegenError::OpNotDefined(_, span) => *span,
            CodegenError::FnDefValidationFailed(_, span) => *span,
            CodegenError::FnPatternUncovered(_, span) => *span,
            _ => None,
        }
    }
}

impl From<inkwell::builder::BuilderError> for CodegenError {
    fn from(e: inkwell::builder::BuilderError) -> Self {
        CodegenError::Unknown(format!("{:?}", e))
    }
}
