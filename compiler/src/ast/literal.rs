use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::literal::Literal;

impl Compile<()> for Literal {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        todo!()
    }
}
