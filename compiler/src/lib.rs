#[cfg(test)]
pub mod tests;

pub mod compiler;
pub mod error;

pub use compiler::{CompileResult, Compiler};
pub use error::CompileError;
