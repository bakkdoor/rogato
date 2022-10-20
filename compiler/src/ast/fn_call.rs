use crate::{Compile, CompileError, CompileResult, Compiler};
use inkwell::values::{BasicMetadataValueEnum, FloatValue};
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

impl<'ctx, 'a> Compile<'ctx, FloatValue<'ctx>> for FnCall<'a> {
    fn compile(&self, c: &'ctx mut Compiler<'ctx>) -> CompileResult<'ctx, FloatValue<'ctx>> {
        match c.get_function(self.id.as_str()) {
            Some(fun) => {
                let mut compiled_args = Vec::with_capacity(self.args.len());

                let mut comp = c;
                for arg in self.args.iter() {
                    let (c, val) = arg.compile(comp)?;
                    compiled_args.push(val);
                    comp = c;
                }

                let argsv: Vec<BasicMetadataValueEnum> = compiled_args
                    .iter()
                    .by_ref()
                    .map(|&val| val.into())
                    .collect();

                match comp
                    .builder()
                    .build_call(fun, argsv.as_slice(), "tmp")
                    .try_as_basic_value()
                    .left()
                {
                    Some(value) => Ok((comp, value.into_float_value())),
                    None => comp.unknown_error("Invalid call produced."),
                }
            }
            None => Err(CompileError::FnNotDefined(self.id.clone())),
        }
    }
}
