use rogato_common::ast::query::{Query, QueryGuards};

use crate::{EvalContext, EvalError, Evaluate, ValueRef};

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
        for guard in self.iter() {
            results.push(guard.evaluate(context)?)
        }
        Ok(results)
    }
}
