use rogato_parser::{parse_ast, ParserContext};

use crate::Compiler;

#[test]
fn compile_add3() {
    let mut parser_ctx = ParserContext::new();
    let context = Compiler::new_context();
    let mut compiler = Compiler::new_with_module_name(&context, "compile_test");

    let func_name = "add3";
    let func_def = parse_ast("let add3 x y z = (x + y) + z", &mut parser_ctx).unwrap();
    compiler.compile_ast(func_def.as_ref()).unwrap();
    let ee = compiler.execution_engine();

    unsafe {
        let function = ee
            .get_function::<unsafe extern "C" fn(f32, f32, f32) -> f32>(func_name)
            .unwrap();

        let params_and_results = [
            ((0.0, 0.0, 0.0), 0.0),
            ((1.0, 0.0, 0.0), 1.0),
            ((1.0, 1.0, 0.0), 2.0),
            ((0.0, 0.0, 42.69), 42.69),
            ((1.0, 2.0, 3.0), 6.0),
            ((0.5, 10.0, 2.5), 13.0),
        ];

        for ((x, y, z), result) in params_and_results {
            let val = function.call(x, y, z);
            assert_eq!(val, result);
            println!("{}({}, {}, {}) = {}", func_name, x, y, z, val);
        }
    }
}

#[test]
fn compile_add2_mul() {
    let func_name = "add2_mul";
    let mut parser_ctx = ParserContext::new();

    let func_def = parse_ast("let add2_mul x y z = (x + y) * z", &mut parser_ctx).unwrap();

    let context = Compiler::new_context();
    let mut compiler = Compiler::new_with_module_name(&context, "compile_test");
    compiler.compile_ast(func_def.as_ref()).unwrap();
    let ee = compiler.execution_engine();

    unsafe {
        let function = ee
            .get_function::<unsafe extern "C" fn(f32, f32, f32) -> f32>(func_name)
            .unwrap();

        let params_and_results = [
            ((1.1, 2.22, 3.333), 11.06556),
            ((1.0, 2.0, 3.0), 9.0),
            ((0.0, 0.0, 0.0), 0.0),
            ((1.0, 0.0, 0.0), 0.0),
            ((0.0, 2.2, 0.0), 0.0),
            ((0.0, 0.0, 3.3), 0.0),
            ((0.5, 10.0, 2.5), 26.25),
        ];

        for ((x, y, z), result) in params_and_results {
            let val = function.call(x, y, z);
            assert_eq!(val, result);
            println!("{}({}, {}, {}) = {}", func_name, x, y, z, val);
        }
    }
}
