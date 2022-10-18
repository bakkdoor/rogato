#[cfg(test)]
pub mod tests;

use inkwell::{context::Context, module::Module};
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
pub struct Compiler<'ctx> {
    context: &'ctx Context,

    #[allow(dead_code)]
    module: Module<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> Self {
        Self { context, module }
    }

    pub fn new_with_module_name<S: ToString>(context: &'ctx Context, module_name: S) -> Self {
        Self {
            context,
            module: context.create_module(module_name.to_string().as_str()),
        }
    }

    pub fn new_context() -> Context {
        Context::create()
    }

    pub fn new_module(&self, name: &str) -> Module<'ctx> {
        self.context.create_module(name)
    }

    pub fn lookup_var(&self) -> Option<Expression> {
        None
    }
}
