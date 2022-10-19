use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::program::Program;

impl Compile<()> for Program {
    fn compile(&self, compiler: &mut Compiler) -> CompilerResult<()> {
        for ast in self.iter() {
            ast.compile(compiler)?;
        }
        Ok(())
    }
}
