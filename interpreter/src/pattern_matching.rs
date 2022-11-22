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

pub enum PatternMatch {
    Matched(ValueRef),
    TryNextPattern,
}

impl PatternMatch {
    pub fn matched(&self) -> bool {
        match self {
            Self::Matched(_) => true,
            Self::TryNextPattern => false,
        }
    }

    pub fn match_failed(&self) -> bool {
        !self.matched()
    }

    pub fn map(&self, f: fn(ValueRef) -> ValueRef) -> Self {
        match self {
            Self::Matched(val) => Self::Matched(f(ValueRef::clone(val))),
            Self::TryNextPattern => Self::TryNextPattern,
        }
    }
}

pub trait PatternMatching {
    fn pattern_match(
        &self,
        eval_context: &mut crate::EvalContext,
        value: ValueRef,
    ) -> Result<PatternMatch, PatternMatchingError>;
}

impl PatternMatching for Pattern {
    fn pattern_match(
        &self,
        context: &mut EvalContext,
        value: ValueRef,
    ) -> Result<PatternMatch, PatternMatchingError> {
        match (self, &*value) {
            (Pattern::Any, _) => Ok(PatternMatch::Matched(value)),
            (Pattern::Var(id), _) => {
                context.define_var(id, ValueRef::clone(&value));
                Ok(PatternMatch::Matched(value))
            }

            (Pattern::EmptyList, Value::List(list)) => {
                if list.is_empty() {
                    Ok(PatternMatch::Matched(value))
                } else {
                    Ok(PatternMatch::TryNextPattern)
                }
            }

            (Pattern::ListCons(head, tail), Value::List(list)) => {
                if list.is_empty() {
                    return Ok(PatternMatch::TryNextPattern);
                }

                head.pattern_match(context, list.head().unwrap())?;
                tail.pattern_match(context, list.tail().into())?;

                Ok(PatternMatch::Matched(value))
            }

            (Pattern::List(patterns), Value::List(items)) => {
                if patterns.len() != items.len() {
                    return Ok(PatternMatch::TryNextPattern);
                }

                for (pat, val) in patterns.iter().zip(items.iter()) {
                    if pat
                        .pattern_match(context, ValueRef::clone(val))?
                        .match_failed()
                    {
                        return Ok(PatternMatch::TryNextPattern);
                    }
                }

                Ok(PatternMatch::Matched(value))
            }

            (Pattern::Tuple(len_p, patterns), Value::Tuple(len, items)) => {
                if len_p != len {
                    return Ok(PatternMatch::TryNextPattern);
                }

                for (pat, val) in patterns.iter().zip(items.iter()) {
                    if pat
                        .pattern_match(context, ValueRef::clone(val))?
                        .match_failed()
                    {
                        return Ok(PatternMatch::TryNextPattern);
                    };
                }

                Ok(PatternMatch::Matched(value))
            }

            (Pattern::Map(kv_pairs_p), Value::Map(map)) => {
                if kv_pairs_p.len() != map.len() {
                    return Ok(PatternMatch::TryNextPattern);
                }

                for kv_pair_p in kv_pairs_p.iter() {
                    let mut matched_pair = false;
                    let (key_p, val_p) = kv_pair_p.pair();
                    for (key, val) in map.iter() {
                        match (
                            key_p.pattern_match(context, ValueRef::clone(key))?,
                            val_p.pattern_match(context, ValueRef::clone(val))?,
                        ) {
                            (PatternMatch::Matched(_), PatternMatch::Matched(_)) => {
                                matched_pair = true;
                                break;
                            }
                            _ => continue,
                        }
                    }

                    if !matched_pair {
                        return Ok(PatternMatch::TryNextPattern);
                    }
                }

                Ok(PatternMatch::Matched(value))
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
                            (PatternMatch::Matched(_), PatternMatch::Matched(_)) => {
                                matched_pair = true;
                                rest_items = rest_items.remove(key);
                                break;
                            }
                            _ => continue,
                        }
                    }

                    if !matched_pair {
                        return Ok(PatternMatch::TryNextPattern);
                    }
                }

                match rest_p.pattern_match(context, rest_items.into())? {
                    PatternMatch::Matched(_) => Ok(PatternMatch::Matched(value)),
                    PatternMatch::TryNextPattern => Ok(PatternMatch::TryNextPattern),
                }
            }

            (Pattern::Bool(pat), Value::Bool(bool)) => {
                if pat == bool {
                    Ok(PatternMatch::Matched(value))
                } else {
                    Ok(PatternMatch::TryNextPattern)
                }
            }

            (Pattern::Number(pat), Value::Number(number)) => {
                if pat == number {
                    Ok(PatternMatch::Matched(value))
                } else {
                    Ok(PatternMatch::TryNextPattern)
                }
            }

            (Pattern::String(pat), Value::String(string)) => {
                if pat == string {
                    Ok(PatternMatch::Matched(value))
                } else {
                    Ok(PatternMatch::TryNextPattern)
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
