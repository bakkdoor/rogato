use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::if_else::IfElse;

impl Compile<()> for IfElse {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        todo!()
    }
}
