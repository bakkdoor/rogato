use crate::{Compile, CompileResult, Compiler};
use inkwell::values::FloatValue;
use rogato_common::ast::module_def::ModuleDef;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for ModuleDef {
    fn compile(&self, _compiler: &mut Compiler<'ctx>) -> CompileResult<'ctx, FloatValue<'ctx>> {
        todo!()
    }
}
