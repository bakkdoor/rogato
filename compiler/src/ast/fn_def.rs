use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::fn_def::FnDef;

impl Compile<()> for FnDef {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        todo!()
    }
}
