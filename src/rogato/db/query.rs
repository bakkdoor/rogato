use crate::rogato::{
    ast::{
        expression::Query,
        query::{QueryBindingError, QueryGuardError},
    },
    interpreter::{EvalContext, Evaluate},
};

use super::val;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryPlanner {}

#[derive(Clone, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum QueryError {
    Unknown(String),
    QueryGuardsFailed(Vec<QueryGuardError>),
    QueryBindingFailed(QueryBindingError),
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
                Err(e) => return Err(QueryError::QueryBindingFailed(e)),
            }
        }

        let guard_results = query.guards().evaluate(&mut query_ctx);
        let guard_failures = Vec::from_iter(
            guard_results
                .iter()
                .filter(|gf| gf.is_err())
                .map(|gf| -> QueryGuardError { gf.clone().err().unwrap() }),
        );

        if guard_failures.is_empty() {
            Ok(query.production().evaluate(&mut query_ctx))
        } else {
            Err(QueryError::QueryGuardsFailed(guard_failures))
        }
    }
}
