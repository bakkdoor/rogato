use std::rc::Rc;

use crate::EvalContext;
use rogato_common::{
    ast::pattern::Pattern,
    val::{Value, ValueRef},
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum PatternBindingError {
    #[error("Unknown PatternBindingError: {0}")]
    Unknown(String),

    #[error("Failed to match pattern: {0} with value: {1}")]
    MatchFail(Pattern, ValueRef),
}

pub trait AttemptBinding {
    fn attempt_binding(
        &self,
        eval_context: &mut crate::EvalContext,
        value: ValueRef,
    ) -> Result<Option<ValueRef>, PatternBindingError>;
}

impl AttemptBinding for Pattern {
    fn attempt_binding(
        &self,
        context: &mut EvalContext,
        value: ValueRef,
    ) -> Result<Option<ValueRef>, PatternBindingError> {
        match (self, &*value) {
            (Pattern::AnyPattern, _) => Ok(Some(value)),
            (Pattern::Var(id), _) => {
                context.define_var(id, Rc::clone(&value));
                Ok(Some(value))
            }

            (Pattern::EmptyList, Value::List(list)) => {
                if list.is_empty() {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            (Pattern::ListCons(head, tail), Value::List(list)) => {
                if list.is_empty() {
                    return Ok(None);
                }
                if let Some(h) = list.head() {
                    head.attempt_binding(context, h)?;
                }
                tail.attempt_binding(context, list.tail().into())?;

                Ok(Some(value))
            }

            (_, _) => Err(PatternBindingError::MatchFail(
                self.clone(),
                Rc::clone(&value),
            )),
        }
    }
}
