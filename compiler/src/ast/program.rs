use crate::{Compile, CompileResult, Compiler};
use rogato_common::ast::program::Program;

impl<'ctx> Compile<'ctx, ()> for Program {
    fn compile(&self, compiler: &'ctx mut Compiler<'ctx>) -> CompileResult<'ctx, ()> {
        let mut comp = compiler;
        for ast in self.iter() {
            (comp, _) = ast.compile(comp)?;
        }
        Ok((comp, ()))
    }
}
