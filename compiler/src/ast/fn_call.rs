use crate::{Compile, Compiler, CompilerResult};
use rogato_common::ast::{fn_call::FnCallArgs, Identifier};

pub struct FnCall<'a> {
    id: &'a Identifier,
    args: &'a FnCallArgs,
}

impl<'a> FnCall<'a> {
    pub fn new(id: &'a Identifier, args: &'a FnCallArgs) -> FnCall<'a> {
        Self { id, args }
    }
}

impl<'a> Compile<()> for FnCall<'a> {
    fn compile(&self, _compiler: &mut Compiler) -> CompilerResult<()> {
        println!("Compiling function call: {}( {} )", self.id, self.args);
        todo!()
    }
}
