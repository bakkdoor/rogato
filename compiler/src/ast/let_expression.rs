use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::let_expression::LetExpression;

impl Compile<()> for LetExpression {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        todo!()
    }
}
