use crate::{Compile, Compiler, CompilerResult};
use inkwell::values::FloatValue;
use rogato_common::ast::let_expression::LetExpression;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for LetExpression {
    fn compile(&self, _compiler: &'ctx mut Compiler) -> CompilerResult<FloatValue<'ctx>> {
        todo!()
    }
}
