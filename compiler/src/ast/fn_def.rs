use crate::{Compile, Compiler, CompilerResult};
use inkwell::types::BasicMetadataTypeEnum;
use rogato_common::ast::fn_def::FnDef;

impl Compile<()> for FnDef {
    fn compile(&self, c: &mut Compiler) -> CompilerResult<()> {
        let f32_type = c.context.f32_type();
        let fn_type = f32_type.fn_type(
            &[
                BasicMetadataTypeEnum::FloatType(f32_type),
                BasicMetadataTypeEnum::FloatType(f32_type),
                BasicMetadataTypeEnum::FloatType(f32_type),
            ],
            false,
        );
        let func_name = self.id();
        let func = c.module.add_function(func_name.as_str(), fn_type, None);

        let basic_block = c.context.append_basic_block(func, self.id());
        c.builder.position_at_end(basic_block);

        let x = func.get_nth_param(0).unwrap().into_float_value();
        let y = func.get_nth_param(1).unwrap().into_float_value();
        let z = func.get_nth_param(2).unwrap().into_float_value();

        let sum = c.builder.build_float_add(x, y, "sum1");
        let sum = c.builder.build_float_add(sum, z, "sum2");

        c.builder.build_return(Some(&sum));

        unsafe {
            let test_fn = c
                .execution_engine
                .get_function::<unsafe extern "C" fn(f32, f32, f32) -> f32>(func_name.as_str())
                .unwrap();

            println!("calling JIT func: {:?}", test_fn);
            let return_value = test_fn.call(1.1f32, 2.22f32, 3.333f32);
            assert_eq!(return_value, 6.653f32);
        }

        Ok(())
    }
}
