use rogato_common::ast::type_expression::TypeDef;

use crate::{Compile, Compiler, CompilerResult};

impl Compile<()> for TypeDef {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        todo!()
    }
}
