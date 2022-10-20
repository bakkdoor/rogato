use crate::{Compile, Compiler, CompilerResult};
use inkwell::values::FloatValue;
use rogato_common::ast::module_def::ModuleDef;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for ModuleDef {
    fn compile(&self, _compiler: &'ctx mut Compiler) -> CompilerResult<FloatValue<'ctx>> {
        todo!()
    }
}
