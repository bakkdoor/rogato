use crate::{EvalContext, EvalError, Evaluate, ValueRef};

use rogato_common::ast::{expression::Query, query::QueryBinding};
use thiserror::Error;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct QueryPlanner {}

#[derive(Error, Debug, PartialEq, Eq, Clone)]

pub enum QueryError {
    #[error("Unknown QueryError: {0}")]
    Unknown(String),

    #[error("Query guard failed: {0}")]
    GuardFailed(Box<EvalError>),

    #[error("Query binding failed: {0:?}")]
    BindingFailed(QueryBindingError),

    #[error("Query production failed: {0}")]
    ProductionFailed(Box<EvalError>),
}

impl From<EvalError> for QueryError {
    fn from(e: EvalError) -> Self {
        Self::GuardFailed(Box::new(e))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QueryBindingError {
    BindingFailed(QueryBinding),
    BindingFailedWith(QueryBinding, Box<EvalError>),
}

pub type QueryResult = Result<ValueRef, QueryError>;

impl QueryPlanner {
    pub fn new() -> QueryPlanner {
        QueryPlanner {}
    }

    pub fn query(&self, context: &mut EvalContext, query: &Query) -> QueryResult {
        let mut query_ctx = context.with_child_env();

        // TODO:
        // - convert bindings into db queries
        // - run each query guard and check its return value for truthiness
        // - return query production if all guards hold

        for binding in query.bindings().iter() {
            match self.attempt_binding(binding, &mut query_ctx) {
                Ok(_) => {}
                Err(e) => return Err(QueryError::BindingFailed(e)),
            }
        }

        query.guards().evaluate(&mut query_ctx)?;

        query
            .production()
            .evaluate(&mut query_ctx)
            .map_err(|e| QueryError::ProductionFailed(Box::new(e)))
    }

    pub fn attempt_binding(
        &self,
        binding: &QueryBinding,
        context: &mut EvalContext,
    ) -> Result<ValueRef, QueryBindingError> {
        match binding.val().evaluate(context) {
            Ok(value) => {
                // todo: actual query logic needed here
                if value.is_none() {
                    if !binding.is_negated() {
                        return Err(QueryBindingError::BindingFailed(binding.clone()));
                    }
                } else if binding.is_negated() {
                    return Err(QueryBindingError::BindingFailed(binding.clone()));
                }

                for id in binding.ids().iter() {
                    context.define_var(id, value.clone())
                }

                Ok(value)
            }
            Err(e) => Err(QueryBindingError::BindingFailedWith(
                binding.clone(),
                Box::new(e),
            )),
        }
    }
}
