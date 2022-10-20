use rogato_common::ast::Identifier;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CompileError {
    #[error("Unknown compiler error: {0}")]
    Unknown(String),

    #[error("Could not find a matching variable with name: {0}")]
    VarNotFound(Identifier),

    #[error("Root comment ignored: {0}")]
    IgnoredRootComment(String),

    #[error("Function not defined: {0}")]
    FnNotDefined(Identifier),

    #[error("Operator not defined: {0}")]
    OpNotDefined(Identifier),

    #[error("FnDef compilation validation failed for: {0}")]
    FnDefValidationFailed(Identifier),
}
