use std::{borrow::Borrow, rc::Rc};

use rogato_common::{
    ast::{
        expression::Expression,
        query::{Query, QueryGuards},
    },
    val::Value,
};

use crate::{query_planner::QueryError, EvalContext, EvalError, Evaluate, ValueRef};

impl Evaluate<ValueRef> for Query {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match context.schedule_query(self) {
            Ok(val) => Ok(val),
            Err(e) => Err(EvalError::from(e)),
        }
    }
}

impl Evaluate<Vec<ValueRef>> for QueryGuards {
    fn evaluate(&self, context: &mut EvalContext) -> Result<Vec<ValueRef>, EvalError> {
        let mut results = vec![];
        for guard_expr in self.iter() {
            results.push(QueryGuard::new(Rc::clone(guard_expr)).evaluate(context)?)
        }
        Ok(results)
    }
}

struct QueryGuard {
    guard_expr: Rc<Expression>,
}

impl QueryGuard {
    pub fn new(guard_expr: Rc<Expression>) -> Self {
        Self { guard_expr }
    }
}

impl Evaluate<ValueRef> for QueryGuard {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let result_value = self.guard_expr.evaluate(context)?;

        match result_value.borrow() {
            Value::Bool(false) => {
                Err(QueryError::GuardConditionFalse(Rc::clone(&self.guard_expr)).into())
            }
            Value::Option(None) => {
                Err(QueryError::GuardConditionNone(Rc::clone(&self.guard_expr)).into())
            }
            _ => Ok(result_value),
        }
    }
}
