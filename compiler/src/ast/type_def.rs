use crate::{Compile, CompileResult, Compiler};
use inkwell::values::FloatValue;
use rogato_common::ast::type_expression::TypeDef;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for TypeDef {
    fn compile(&self, _compiler: &mut Compiler<'ctx>) -> CompileResult<'ctx, FloatValue<'ctx>> {
        todo!()
    }
}
