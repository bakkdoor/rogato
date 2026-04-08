pub mod environment;
pub mod error;
pub mod inferred_type;
pub mod inferrer;
pub(crate) mod pattern;
pub mod type_check;

#[cfg(test)]
pub mod tests;

pub use environment::{FnSignature, TypeEnvironment};
pub use error::TypeCheckError;
pub use inferred_type::InferredType;
pub use inferrer::TypeInferrer;
pub use type_check::TypeCheck;
