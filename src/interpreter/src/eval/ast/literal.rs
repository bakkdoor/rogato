use std::fmt::Display;

use crate::eval::{EvalContext, EvalError, Evaluate};
use rogato_common::{
    ast::literal::{Literal, TupleItems},
    val::{self, ValueRef},
};

impl Evaluate<ValueRef> for Literal {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match self {
            Literal::Int64(number) => Ok(val::int64(*number)),
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
