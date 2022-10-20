use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::lambda::Lambda;

impl Compile<()> for Lambda {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        todo!()
    }
}
