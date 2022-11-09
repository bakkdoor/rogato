use std::rc::Rc;

use crate::{EvalContext, Identifier};
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
    MatchFailed(Pattern, ValueRef),

    #[error("Failed to match fn {0} variant for pattern: {1:?} with value: {2:?}")]
    NoFnVariantMatched(Identifier, Option<Rc<Pattern>>, Vec<ValueRef>),
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
            (Pattern::Any, _) => Ok(Some(value)),
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

                head.attempt_binding(context, list.head().unwrap())?;
                tail.attempt_binding(context, list.tail().into())?;

                Ok(Some(value))
            }

            (Pattern::List(patterns), Value::List(items)) => {
                if patterns.len() != items.len() {
                    return Ok(None);
                }

                for (pat, val) in patterns.iter().zip(items.iter()) {
                    if pat.attempt_binding(context, Rc::clone(val))?.is_none() {
                        return Ok(None);
                    }
                }

                Ok(Some(value))
            }

            (Pattern::Tuple(len_p, patterns), Value::Tuple(len, items)) => {
                if len_p != len {
                    return Ok(None);
                }

                for (pat, val) in patterns.iter().zip(items.iter()) {
                    if pat.attempt_binding(context, Rc::clone(val))?.is_none() {
                        return Ok(None);
                    };
                }

                Ok(Some(value))
            }

            (Pattern::Bool(pat), Value::Bool(bool)) => {
                if pat == bool {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            (Pattern::Number(pat), Value::Number(number)) => {
                if pat == number {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            (Pattern::String(pat), Value::String(string)) => {
                if pat == string {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            (_, _) => Err(PatternBindingError::MatchFailed(
                self.clone(),
                Rc::clone(&value),
            )),
        }
    }
}