use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::query::Query;

impl Compile<()> for Query {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        todo!()
    }
}
