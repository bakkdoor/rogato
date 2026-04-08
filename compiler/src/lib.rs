#[cfg(test)]
pub mod tests;

pub mod codegen;
pub mod error;

pub use codegen::{Codegen, CodegenResult};
pub use error::CodegenError;
