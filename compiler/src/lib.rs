#[cfg(test)]
pub mod tests;

use inkwell::{
    builder::Builder, context::Context, execution_engine::ExecutionEngine, module::Module,
    OptimizationLevel,
};
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
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    #[allow(dead_code)]
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> Self {
        let execution_engine = Compiler::execution_engine(&module);
        Self {
            context,
            module,
            builder: context.create_builder(),
            execution_engine,
        }
    }

    pub fn new_with_module_name<S: ToString>(context: &'ctx Context, module_name: S) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module_name.to_string().as_str());
        let execution_engine = Compiler::execution_engine(&module);
        Self {
            context,
            module,
            builder,
            execution_engine,
        }
    }

    pub fn execution_engine(module: &Module<'ctx>) -> ExecutionEngine<'ctx> {
        module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap()
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
