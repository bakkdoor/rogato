#[cfg(test)]
pub mod tests;

pub mod ast;
pub mod compiler;
pub mod error;

use std::{ops::Deref, rc::Rc};

pub use compiler::Compiler;
pub use error::CompileError;

pub trait Compile<'ctx, T> {
    fn compile(&self, compiler: &'ctx mut Compiler<'ctx>) -> CompileResult<'ctx, T>;
}

impl<'ctx, T, Type: Compile<'ctx, T>> Compile<'ctx, T> for Rc<Type> {
    fn compile(&self, compiler: &'ctx mut Compiler<'ctx>) -> CompileResult<'ctx, T> {
        self.deref().compile(compiler)
    }
}

pub type CompileResult<'ctx, T> = Result<(&'ctx mut Compiler<'ctx>, T), CompileError>;
