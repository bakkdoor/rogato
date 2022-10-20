use crate::{Compile, CompileResult, Compiler};
use inkwell::values::FloatValue;
use rogato_common::ast::literal::Literal;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for Literal {
    fn compile(&self, _compiler: &mut Compiler<'ctx>) -> CompileResult<'ctx, FloatValue<'ctx>> {
        todo!()
    }
}
