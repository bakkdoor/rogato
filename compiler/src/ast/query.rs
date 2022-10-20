use crate::{Compile, CompileResult, Compiler};
use inkwell::values::FloatValue;
use rogato_common::ast::query::Query;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for Query {
    fn compile(&self, _compiler: &mut Compiler<'ctx>) -> CompileResult<'ctx, FloatValue<'ctx>> {
        todo!()
    }
}
