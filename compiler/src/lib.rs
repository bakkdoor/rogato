#[cfg(test)]
pub mod tests;

use rogato_common::ast::expression::Expression;
use thiserror::Error;

pub mod ast;

pub trait Compile<T> {
    fn compile(&self, compiler: &mut Compiler) -> CompilerResult<T>;
}

pub type CompilerResult<T> = Result<T, CodeGenError>;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CodeGenError {
    #[error("Unknown codegen error: {0}")]
    Unknown(String),
}

#[derive(Debug)]
pub struct Compiler {
    llvm: inkwell::context::Context,
}

impl Compiler {
    pub fn new() -> Self {
        let compiler = Self {
            llvm: inkwell::context::Context::create(),
        };
        println!("New compiler with llvm context: {:?}", compiler.llvm);
        compiler
    }

    pub fn lookup_var(&self) -> Option<Expression> {
        None
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler::new()
    }
}
