use crate::{Compile, Compiler, CompilerResult};
use inkwell::values::FloatValue;
use rogato_common::ast::literal::Literal;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for Literal {
    fn compile(&self, _compiler: &'ctx mut Compiler) -> CompilerResult<FloatValue<'ctx>> {
        todo!()
    }
}
