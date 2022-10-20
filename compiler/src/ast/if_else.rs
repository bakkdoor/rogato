use crate::{Compile, CompileResult, Compiler};
use inkwell::values::FloatValue;
use rogato_common::ast::if_else::IfElse;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for IfElse {
    fn compile(&self, _c: &mut Compiler<'ctx>) -> CompileResult<'ctx, FloatValue<'ctx>> {
        todo!()
    }
}
