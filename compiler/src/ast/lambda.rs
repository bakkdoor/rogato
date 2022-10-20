use crate::{Compile, CompileResult, Compiler};
use inkwell::values::FloatValue;
use rogato_common::ast::lambda::Lambda;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for Lambda {
    fn compile(&self, _compiler: &mut Compiler<'ctx>) -> CompileResult<'ctx, FloatValue<'ctx>> {
        todo!()
    }
}
