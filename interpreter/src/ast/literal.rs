use std::{fmt::Display, ops::Deref};

use crate::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::literal::{Literal, TupleItems},
    val::{self, Value, ValueRef},
};

impl Evaluate<ValueRef> for Literal {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match self {
            Literal::Number(number) => Ok(val::number(*number)),
            Literal::String(string) => Ok(val::string(string)),
            Literal::Tuple(items) => {
                let mut values = vec![];
                for item in items.iter() {
                    values.push(item.evaluate(context)?)
                }
                Ok(val::tuple(values))
            }
            Literal::List(items) => {
                let mut values = vec![];
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
                let mut prop_values = vec![];
                for (id, expr) in props.iter() {
                    prop_values.push((id.clone(), expr.evaluate(context)?))
                }
                Ok(val::object(prop_values))
            }
        }
    }
}

impl<T: Evaluate<ValueRef> + Display> Evaluate<ValueRef> for TupleItems<T> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut values = vec![];
        for item in self.iter() {
            values.push(item.evaluate(context)?)
        }
        Ok(val::list(values))
    }
}
