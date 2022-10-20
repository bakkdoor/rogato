use std::collections::HashMap;

use crate::{Compile, CompileResult, Compiler};
use inkwell::{types::BasicMetadataTypeEnum, values::FloatValue};
use rogato_common::ast::{
    fn_def::{FnDef, FnDefBody},
    Identifier,
};

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for FnDef {
    fn compile(&self, c: &'ctx mut Compiler) -> CompileResult<FloatValue<'ctx>> {
        let f32_type = c.context().f32_type();
        let fn_type = f32_type.fn_type(
            &[
                BasicMetadataTypeEnum::FloatType(f32_type),
                BasicMetadataTypeEnum::FloatType(f32_type),
                BasicMetadataTypeEnum::FloatType(f32_type),
            ],
            false,
        );
        let func_name = self.id();
        let func = c.module().add_function(func_name.as_str(), fn_type, None);
        c.set_fn_value(func);

        let basic_block = c.context().append_basic_block(func, func_name);
        c.builder().position_at_end(basic_block);

        let args_with_idx: HashMap<Identifier, u32> = HashMap::from_iter(
            self.args()
                .iter()
                .cloned()
                .zip(0..self.args().len().try_into().unwrap()),
        );

        for (arg, idx) in args_with_idx.iter() {
            let pointer_val = c.create_entry_block_alloca(arg.as_str());
            c.store_var(arg.as_str(), pointer_val);

            c.builder().build_store(
                pointer_val,
                func.get_nth_param(*idx).unwrap().into_float_value(),
            );
        }

        match self.body().as_ref() {
            FnDefBody::RogatoFn(expr) => {
                let builder = c.builder();
                let body = expr.compile(c)?;
                builder.build_return(Some(&body));
                Ok(body)
            }
            _ => c.unknown_error("Cannot compile function with NativeFn body!"),
        }
    }
}
