use crate::rogato::{
    ast::{expression::Query, query::QueryBindingError},
    interpreter::{EvalContext, EvalError, Evaluate},
};

use super::val;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryPlanner {}

#[derive(Clone, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum QueryError {
    Unknown(String),
    GuardFailed(Box<EvalError>),
    BindingFailed(QueryBindingError),
    ProductionFailed(Box<EvalError>),
}

impl From<EvalError> for QueryError {
    fn from(e: EvalError) -> Self {
        Self::GuardFailed(Box::new(e))
    }
}

pub type QueryResult = Result<val::Value, QueryError>;

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
            match binding.attempt_binding(&mut query_ctx) {
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
}
