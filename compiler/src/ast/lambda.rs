use crate::{Compile, Compiler, CompilerResult};
use inkwell::values::FloatValue;
use rogato_common::ast::lambda::Lambda;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for Lambda {
    fn compile(&self, _compiler: &'ctx mut Compiler) -> CompilerResult<FloatValue<'ctx>> {
        todo!()
    }
}
