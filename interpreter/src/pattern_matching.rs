use std::rc::Rc;

use crate::{EvalContext, Identifier};
use rogato_common::{
    ast::pattern::Pattern,
    val::{Map, Value, ValueRef},
};
use thiserror::Error;

type FuncId = Identifier;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum PatternMatchingError {
    #[error("Unknown PatternMatchingError in {0} : {1}")]
    Unknown(FuncId, String),

    #[error("Failed to match pattern in {0} : {1} with value: {2}")]
    MatchFailed(FuncId, Pattern, ValueRef),

    #[error("Failed to match fn {0} variant for pattern: {1:?} with value: {2:?}")]
    NoFnVariantMatched(FuncId, Option<Rc<Pattern>>, Vec<ValueRef>),
}

pub trait PatternMatching {
    fn pattern_match(
        &self,
        eval_context: &mut crate::EvalContext,
        value: ValueRef,
    ) -> Result<Option<ValueRef>, PatternMatchingError>;
}

impl PatternMatching for Pattern {
    fn pattern_match(
        &self,
        context: &mut EvalContext,
        value: ValueRef,
    ) -> Result<Option<ValueRef>, PatternMatchingError> {
        match (self, &*value) {
            (Pattern::Any, _) => Ok(Some(value)),
            (Pattern::Var(id), _) => {
                context.define_var(id, ValueRef::clone(&value));
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

                head.pattern_match(context, list.head().unwrap())?;
                tail.pattern_match(context, list.tail().into())?;

                Ok(Some(value))
            }

            (Pattern::List(patterns), Value::List(items)) => {
                if patterns.len() != items.len() {
                    return Ok(None);
                }

                for (pat, val) in patterns.iter().zip(items.iter()) {
                    if pat.pattern_match(context, ValueRef::clone(val))?.is_none() {
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
                    if pat.pattern_match(context, ValueRef::clone(val))?.is_none() {
                        return Ok(None);
                    };
                }

                Ok(Some(value))
            }

            (Pattern::Map(kv_pairs_p), Value::Map(map)) => {
                if kv_pairs_p.len() != map.len() {
                    return Ok(None);
                }

                for kv_pair_p in kv_pairs_p.iter() {
                    let mut matched_pair = false;
                    let (key_p, val_p) = kv_pair_p.pair();
                    for (key, val) in map.iter() {
                        match (
                            key_p.pattern_match(context, ValueRef::clone(key))?,
                            val_p.pattern_match(context, ValueRef::clone(val))?,
                        ) {
                            (Some(_), Some(_)) => {
                                matched_pair = true;
                                break;
                            }
                            _ => continue,
                        }
                    }

                    if !matched_pair {
                        return Ok(None);
                    }
                }

                Ok(Some(value))
            }

            (Pattern::MapCons(kv_pairs_p, rest_p), Value::Map(map)) => {
                let mut rest_items: Map = map.clone();
                for kv_pair_p in kv_pairs_p.iter() {
                    let mut matched_pair = false;
                    let (key_p, val_p) = kv_pair_p.pair();
                    for (key, val) in map.iter() {
                        match (
                            key_p.pattern_match(context, ValueRef::clone(key))?,
                            val_p.pattern_match(context, ValueRef::clone(val))?,
                        ) {
                            (Some(_), Some(_)) => {
                                matched_pair = true;
                                rest_items = rest_items.remove(key);
                                break;
                            }
                            _ => continue,
                        }
                    }

                    if !matched_pair {
                        return Ok(None);
                    }
                }

                match rest_p.pattern_match(context, rest_items.into())? {
                    Some(_) => Ok(Some(value)),
                    None => Ok(None),
                }
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

            (_, _) => Err(PatternMatchingError::MatchFailed(
                context.current_func_id(),
                self.clone(),
                ValueRef::clone(&value),
            )),
        }
    }
}
