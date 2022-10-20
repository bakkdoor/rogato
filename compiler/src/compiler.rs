use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    values::{FunctionValue, PointerValue},
    OptimizationLevel,
};
use std::collections::HashMap;

use crate::error::CompileError;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug)]
pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    #[allow(dead_code)]
    execution_engine: ExecutionEngine<'ctx>,

    fn_value_opt: Option<FunctionValue<'ctx>>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> Self {
        let execution_engine = Compiler::new_execution_engine(&module);
        Self {
            context,
            module,
            builder: context.create_builder(),
            execution_engine,
            fn_value_opt: None,
            variables: HashMap::new(),
        }
    }

    pub fn new_with_module_name<S: ToString>(context: &'ctx Context, module_name: S) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module_name.to_string().as_str());
        let execution_engine = Compiler::new_execution_engine(&module);
        Self {
            context,
            module,
            builder,
            execution_engine,
            fn_value_opt: None,
            variables: HashMap::new(),
        }
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }
    pub fn execution_engine(&self) -> &ExecutionEngine<'ctx> {
        &self.execution_engine
    }

    pub fn new_execution_engine(module: &Module<'ctx>) -> ExecutionEngine<'ctx> {
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

    pub fn lookup_var<S: ToString>(&self, name: S) -> Option<&PointerValue<'ctx>> {
        self.variables.get(&name.to_string())
    }

    pub fn store_var<S: ToString>(&mut self, name: S, pointer_val: PointerValue<'ctx>) {
        self.variables.insert(name.to_string(), pointer_val);
    }

    pub fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.fn_value().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name)
    }

    #[inline]
    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    #[inline]
    pub fn builder(&self) -> &Builder<'ctx> {
        &self.builder
    }

    /// Returns the `FunctionValue` representing the function being compiled.
    #[inline]
    pub fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }

    /// Sets the `FunctionValue` representing the function being compiled.
    #[inline]
    pub fn set_fn_value(&mut self, fn_val: FunctionValue<'ctx>) {
        self.fn_value_opt = Some(fn_val)
    }

    /// Gets a defined function given its name.
    #[inline]
    pub fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    pub fn unknown_error<T, S: ToString>(&self, message: S) -> CompileResult<T> {
        Err(CompileError::Unknown(message.to_string()))
    }
}
