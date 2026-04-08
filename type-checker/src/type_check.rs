use crate::environment::TypeEnvironment;
use crate::error::TypeCheckError;
use crate::inferred_type::InferredType;
use crate::inferrer::TypeInferrer;

use rogato_common::ast::{expression::Expression, fn_def::FnDef};

pub trait TypeCheck<T> {
    fn type_check(&self, context: &mut TypeEnvironment) -> Result<T, TypeCheckError>;
}

impl TypeCheck<InferredType> for Expression {
    fn type_check(&self, context: &mut TypeEnvironment) -> Result<InferredType, TypeCheckError> {
        let mut inferrer = TypeInferrer::with_env(context.clone());
        inferrer.check_expression(self)
    }
}

impl TypeCheck<InferredType> for FnDef {
    fn type_check(&self, context: &mut TypeEnvironment) -> Result<InferredType, TypeCheckError> {
        let inferrer = TypeInferrer::with_env(context.clone());
        Ok(inferrer.infer_fn_def(self))
    }
}
