#[cfg(test)]
pub mod tests;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CodeGenError {
    #[error("Unknown codegen error: {0}")]
    Unknown(String),
}

pub struct CompilerContext {}

pub trait Compile<T> {
    fn compile(&self, context: &mut CompilerContext) -> Result<T, CodeGenError>;
}

pub struct Compiler {}

impl Default for Compiler {
    fn default() -> Self {
        Compiler::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn gen_self_calling_fn(&self) {
        use inkwell::context::Context;

        // A simple function which calls itself:
        let context = Context::create();
        let module = context.create_module("ret");
        let builder = context.create_builder();
        let i32_type = context.i32_type();
        let fn_type = i32_type.fn_type(&[i32_type.into()], false);
        let fn_value = module.add_function("ret", fn_type, None);
        let entry = context.append_basic_block(fn_value, "entry");
        let i32_arg = fn_value.get_first_param().unwrap();
        let _md_string = context.metadata_string("a metadata");

        builder.position_at_end(entry);

        let ret_val = builder
            .build_call(fn_value, &[i32_arg.into()], "call")
            .try_as_basic_value()
            .left()
            .unwrap();

        builder.build_return(Some(&ret_val));
    }
}
