use std::{fmt::Display, ops::Deref};

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::literal::{Literal, TupleItems},
    val::{self, Value, ValueRef},
};

#[cfg(feature = "flame_it")]
use flamer::flame;

impl Evaluate<ValueRef> for Literal {
    #[cfg_attr(feature = "flame_it", flame("Literal::"))]
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match self {
            Literal::Bool(boolean) => Ok(val::bool(*boolean)),
            Literal::Number(number) => Ok(val::number(*number)),
            Literal::String(string) => Ok(val::string(string)),
            Literal::Tuple(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items.iter() {
                    values.push(item.evaluate(context)?)
                }
                Ok(val::tuple(values))
            }
            Literal::List(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items.iter() {
                    values.push(item.evaluate(context)?)
                }
                Ok(val::list(values))
            }
            Literal::ListCons(first, rest) => {
                let value = first.evaluate(context)?;
                let rest = rest.evaluate(context)?;
                match rest.deref() {
                    Value::List(list) => Ok(list.cons(value).into()),
                    _ => Err(EvalError::ListConsInvalidList(rest)),
                }
            }
            Literal::Struct(_struct_id, props) => {
                let mut prop_values = Vec::with_capacity(props.len());
                for (id, expr) in props.iter() {
                    prop_values.push((id.clone(), expr.evaluate(context)?))
                }
                Ok(val::object(prop_values))
            }
            Literal::Map(kv_pairs) => {
                let mut pairs: Vec<(ValueRef, ValueRef)> = Vec::with_capacity(kv_pairs.len());
                for kv_pair in kv_pairs.iter() {
                    pairs.push((
                        kv_pair.key.evaluate(context)?,
                        kv_pair.value.evaluate(context)?,
                    ));
                }
                Ok(val::map(pairs))
            }
            Literal::MapCons(kv_pairs, rest) => {
                let rest = rest.evaluate(context)?;
                match rest.deref() {
                    Value::Map(map) => {
                        let mut pairs: Vec<(ValueRef, ValueRef)> =
                            Vec::with_capacity(kv_pairs.len());

                        for kv_pair in kv_pairs.iter() {
                            pairs.push((
                                kv_pair.key.evaluate(context)?,
                                kv_pair.value.evaluate(context)?,
                            ));
                        }
                        Ok(map.cons(pairs).into())
                    }
                    _ => Err(EvalError::MapConsInvalidMap(rest)),
                }
            }
        }
    }
}

impl<T: Evaluate<ValueRef> + Display> Evaluate<ValueRef> for TupleItems<T> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut values = Vec::with_capacity(self.len());
        for item in self.iter() {
            values.push(item.evaluate(context)?)
        }
        Ok(val::list(values))
    }
}
