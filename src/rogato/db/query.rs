use crate::rogato::ast::expression::Query;

use super::val;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryPlanner {}

#[derive(Clone, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum QueryError {
    Unknown(String),
}

pub type QueryResult = Result<val::Value, QueryError>;

impl QueryPlanner {
    pub fn new() -> QueryPlanner {
        QueryPlanner {}
    }

    pub fn query(&self, _query: &Query) -> QueryResult {
        Ok(val::string("query result (TODO)"))
    }
}
