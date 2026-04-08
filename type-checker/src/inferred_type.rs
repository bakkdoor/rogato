use std::rc::Rc;

use rogato_common::ast::type_expression::TypeExpression;

#[derive(Clone, Debug)]
pub enum InferredType {
    Known(Rc<TypeExpression>),
    Unknown,
}

impl InferredType {
    pub fn known(type_expr: Rc<TypeExpression>) -> Self {
        InferredType::Known(type_expr)
    }

    pub fn unknown() -> Self {
        InferredType::Unknown
    }

    pub fn is_known(&self) -> bool {
        matches!(self, InferredType::Known(_))
    }

    pub fn inner(&self) -> Option<&Rc<TypeExpression>> {
        match self {
            InferredType::Known(te) => Some(te),
            InferredType::Unknown => None,
        }
    }

    pub fn unify(&self, other: &InferredType) -> InferredType {
        match (self, other) {
            (InferredType::Known(a), InferredType::Known(b)) if a == b => self.clone(),
            (InferredType::Unknown, _) => other.clone(),
            (_, InferredType::Unknown) => self.clone(),
            _ => InferredType::Unknown,
        }
    }
}
