use rogato_common::ast::module_def::ModuleDef;

use crate::{Compile, Compiler, CompilerResult};

impl Compile<()> for ModuleDef {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        todo!()
    }
}
