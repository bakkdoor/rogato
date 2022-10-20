use crate::{Compile, Compiler, CompilerResult};
use inkwell::values::FloatValue;
use rogato_common::ast::query::Query;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for Query {
    fn compile(&self, _compiler: &'ctx mut Compiler) -> CompilerResult<FloatValue<'ctx>> {
        todo!()
    }
}
