use rogato_common::ast::Identifier;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CodegenError {
    #[error("Unknown compiler codegen error: {0}")]
    Unknown(String),

    #[error("Compiler feature not yet implemented: {0}")]
    NotYetImplemented(String),

    #[error("Could not find a matching variable with name: {0}")]
    VarNotFound(Identifier),

    #[error("Root comment ignored: {0}")]
    IgnoredRootComment(String),

    #[error("Function not defined: {0}")]
    FnNotDefined(Identifier),

    #[error("Operator not defined: {0}")]
    OpNotDefined(Identifier),

    #[error("FnDef codegen validation failed for: {0}")]
    FnDefValidationFailed(Identifier),

    #[error("Type {0} not yet supported in codegen")]
    UnsupportedLLVMType(String),

    #[error("Cannot convert type {0} to {1}")]
    TypeConversionError(String, String),

    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    #[error("Function '{0}' has uncovered pattern - no catch-all in last variant")]
    FnPatternUncovered(Identifier),
}

impl From<inkwell::builder::BuilderError> for CodegenError {
    fn from(e: inkwell::builder::BuilderError) -> Self {
        CodegenError::Unknown(format!("{:?}", e))
    }
}
