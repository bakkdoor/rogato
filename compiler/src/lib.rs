#[cfg(test)]
pub mod tests;

pub mod ast;
pub mod compiler;
pub mod error;

use std::{ops::Deref, rc::Rc};

pub use compiler::{CompileResult, Compiler};
pub use error::CompileError;

pub trait Compile<'ctx, T> {
    fn compile(&self, compiler: &'ctx mut Compiler) -> CompilerResult<T>;
}

impl<'ctx, T, Type: Compile<'ctx, T>> Compile<'ctx, T> for Rc<Type> {
    fn compile(&self, compiler: &'ctx mut crate::Compiler) -> crate::CompilerResult<T> {
        self.deref().compile(compiler)
    }
}

pub type CompilerResult<T> = Result<T, CompileError>;
