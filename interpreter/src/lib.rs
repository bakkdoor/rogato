#[cfg(test)]
pub mod tests;

pub mod query_planner;

pub mod ast;
pub mod environment;
pub mod eval_context;
pub mod lib_std;
pub mod module;

use ast::pattern::PatternBindingError;
pub use eval_context::EvalContext;
use query_planner::QueryError;
pub use rogato_common::{
    native_fn::{NativeFn, NativeFnError},
    val::{Value, ValueRef},
};

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum EvalError {
    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Import failed, module not found for: {0} {{  }}")]
    ImportFailed(Identifier, Vec<Identifier>),

    #[error("{0}")]
    QueryFailed(QueryError),

    #[error("{0}")]
    FnCallArgumentError(Box<EvalError>),

    #[error("Constant or Type not found: {0}")]
    ConstOrTypeNotFound(Identifier),

    #[error("DB Type not found: {0}")]
    DBTypeNotFound(Identifier),

    #[error("Operator not defined: {0}")]
    OperatorNotDefined(Identifier),

    #[error("Var not defined: {0}")]
    VarNotDefined(Identifier),

    #[error("Function not defined: {0}")]
    FunctionNotDefined(Identifier),

    #[error("Function arity mismatch for {0} : Expected at least: {1} but got: {2}")]
    FunctionArityMismatch(Identifier, usize, usize),

    #[error("Lambda arity mismatch: Expected: {0} but got: {1}")]
    LambdaArityMismatch(usize, usize),

    #[error("{0}")]
    NativeFnFailed(NativeFnError),

    #[error("IfElse condition is not a Bool value: {0}")]
    IFElseConditionNotBool(ValueRef),

    #[error("List cons requires List, was given: {0}")]
    ListConsInvalidList(ValueRef),

    #[error("Map cons requires Map, was given: {0}")]
    MapConsInvalidMap(ValueRef),

    #[error("EvalError during pattern match in {0} : {1}")]
    PatternBindingFailed(Identifier, PatternBindingError),
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
        NativeFnError::EvaluationFailed("EvalError: NativeFnError:".into(), format!("{}", e))
    }
}

impl From<PatternBindingError> for EvalError {
    fn from(e: PatternBindingError) -> Self {
        match &e {
            PatternBindingError::Unknown(func_id, _) => {
                EvalError::PatternBindingFailed(func_id.clone(), e)
            }
            PatternBindingError::MatchFailed(func_id, _, _) => {
                EvalError::PatternBindingFailed(func_id.clone(), e)
            }
            PatternBindingError::NoFnVariantMatched(func_id, _, _) => {
                EvalError::PatternBindingFailed(func_id.clone(), e)
            }
        }
    }
}

pub type Identifier = rogato_common::ast::Identifier;

pub trait Evaluate<T> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<T, EvalError>;
}
