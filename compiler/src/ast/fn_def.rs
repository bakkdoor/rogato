use crate::{Compile, Compiler, CompilerResult};
use inkwell::types::BasicMetadataTypeEnum;
use rogato_common::ast::fn_def::FnDef;

impl Compile<()> for FnDef {
    fn compile(&self, c: &mut Compiler) -> CompilerResult<()> {
        let f32_type = c.context.f32_type();
        let fn_type = f32_type.clone().fn_type(
            &[
                BasicMetadataTypeEnum::FloatType(f32_type),
                BasicMetadataTypeEnum::FloatType(f32_type),
                BasicMetadataTypeEnum::FloatType(f32_type),
            ],
            false,
        );
        let func = c.module.add_function(self.id(), fn_type, None);

        let basic_block = c.context.append_basic_block(func, self.id());
        c.builder.position_at_end(basic_block);

        let x = func.get_nth_param(0).unwrap().into_float_value();
        let y = func.get_nth_param(1).unwrap().into_float_value();
        let z = func.get_nth_param(2).unwrap().into_float_value();

        let sum = c.builder.build_float_add(x, y, "sum1");
        let sum = c.builder.build_float_add(sum, z, "sum2");

        c.builder.build_return(Some(&sum));

        unsafe {
            // c.execution_engine.get_function(self.id().as_str()).ok();
            let test_fn = c
                .execution_engine
                .get_function::<unsafe extern "C" fn(f64, f64, f64) -> f64>(self.id().as_str())
                .unwrap();
            let return_value = test_fn.call(1.1f64, 2.22f64, 3.333f64);
            assert_eq!(return_value, 6.653f64);
        }

        todo!()
    }
}
