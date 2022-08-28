#[cfg(test)]
pub mod tests;

pub mod query_planner;

pub mod ast;
pub mod environment;
pub mod eval_context;
pub mod lib_std;
pub mod module;

pub use eval_context::EvalContext;
use query_planner::QueryError;
pub use rogato_common::{
    native_fn::{NativeFn, NativeFnError},
    val::{Value, ValueRef},
};

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum EvalError {
    #[error("Unknown evaluation error: {0}")]
    Unknown(String),

    #[error("EvalError: QueryError: {0:?}")]
    QueryFailed(QueryError),

    #[error("EvalError: FnCall argument error: {0}")]
    FnCallArgumentError(Box<EvalError>),

    #[error("EvalError: Constant or Type not found: {0}")]
    ConstOrTypeNotFound(Identifier),

    #[error("EvalError: Operator not defined: {0}")]
    OperatorNotDefined(Identifier),

    #[error("EvalError: Var not defined: {0}")]
    VarNotDefined(Identifier),

    #[error("EvalError: Function not defined: {0}")]
    FunctionNotDefined(Identifier),

    #[error("EvalError: Function arity mismatch for {0} : Expected: {1} but got: {2}")]
    FunctionArityMismatch(Identifier, usize, usize),

    #[error("EvalError: Lambda arity mismatch: Expected: {0} but got: {1}")]
    LambdaArityMismatch(usize, usize),

    #[error("EvalError: {0}")]
    NativeFnFailed(NativeFnError),

    #[error("EvalError: IfElse condition is not a Bool value: {0}")]
    IFElseConditionNotBool(ValueRef),
}

impl From<QueryError> for EvalError {
    fn from(qe: QueryError) -> Self {
        EvalError::QueryFailed(qe)
    }
}

impl From<NativeFnError> for EvalError {
    fn from(nfe: NativeFnError) -> Self {
        EvalError::NativeFnFailed(nfe)
    }
}

impl From<EvalError> for NativeFnError {
    fn from(e: EvalError) -> Self {
        NativeFnError::EvaluationFailed(format!("{}", e))
    }
}

pub type Identifier = rogato_common::ast::Identifier;

pub trait Evaluate<T> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<T, EvalError>;
}
