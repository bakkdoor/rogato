use crate::rogato::{
    ast::{expression::Query, query::QueryGuardError},
    interpreter::{EvalContext, Evaluate},
};

use super::val;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryPlanner {}

#[derive(Clone, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum QueryError {
    Unknown(String),
    QueryGuards(Vec<QueryGuardError>),
}

pub type QueryResult = Result<val::Value, QueryError>;

impl QueryPlanner {
    pub fn new() -> QueryPlanner {
        QueryPlanner {}
    }

    pub fn query(&self, context: &mut EvalContext, query: &Query) -> QueryResult {
        let mut query_ctx = context.with_child_env();
        for binding in query.bindings().iter() {
            // TODO:
            // - convert bindings into db queries
            // - run each query guard and check its return value for truthiness
            // - return query production if all guards hold
            let value = binding.value_expr().evaluate(&mut query_ctx);
            // todo: actual query logic needed here
            if value.is_null() {
                if !binding.is_negated() {
                    panic!("query failed: {:?}", query)
                }
            } else if binding.is_negated() {
                panic!("negated query failed: {:?}", query)
            }
            for id in binding.ids().iter() {
                query_ctx.define_var(id, value.clone())
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
            Err(QueryError::QueryGuards(guard_failures))
        }
    }
}
