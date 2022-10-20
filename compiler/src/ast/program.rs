use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::program::Program;

impl<'ctx> Compile<'ctx, ()> for Program {
    fn compile(&self, compiler: &'ctx mut Compiler) -> CompilerResult<()> {
        for ast in self.iter() {
            ast.compile(compiler)?;
        }
        Ok(())
    }
}
