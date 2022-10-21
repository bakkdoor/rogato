use std::rc::Rc;

use rogato_common::ast::{fn_def::FnDef, AST};
use rogato_parser::{parse_ast, ParserContext};

use crate::Compiler;

pub fn parse_fn_def(code: &str) -> Rc<FnDef> {
    let mut parser_ctx = ParserContext::new();
    let ast = parse_ast(code, &mut parser_ctx).unwrap();
    match ast.as_ref() {
        AST::FnDef(f) => Rc::clone(f),
        _ => panic!("Invalid AST node, expected FnDef"),
    }
}

type F32FnType = unsafe extern "C" fn(f32, f32, f32) -> f32;

#[test]
fn compile_add3() {
    let context = Compiler::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compile_test");
    let fpm = Compiler::default_function_pass_manager(&module);
    let ee = Compiler::default_execution_engine(&module);
    let mut compiler = Compiler::new(&context, &module, &builder, &fpm, &ee);

    let func_def = parse_fn_def("let add3 x y z = (x + y) + z");
    compiler.compile_fn_def(func_def.as_ref()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine()
            .get_function::<unsafe extern "C" fn(f32, f32, f32) -> f32>("add3")
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
        }
    }
}

#[test]
fn compile_add2_mul() {
    let context = Compiler::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compile_test");
    let fpm = Compiler::default_function_pass_manager(&module);
    let ee = Compiler::default_execution_engine(&module);
    let mut compiler = Compiler::new(&context, &module, &builder, &fpm, &ee);

    let func_def = parse_fn_def("let add2_mul x y z = (x + y) * z");
    compiler.compile_fn_def(func_def.as_ref()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine()
            .get_function::<F32FnType>("add2_mul")
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
        }
    }
}

#[test]
fn compile_multiple_functions() {
    let context = Compiler::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compile_test");
    let fpm = Compiler::default_function_pass_manager(&module);
    let ee = Compiler::default_execution_engine(&module);
    let mut compiler = Compiler::new(&context, &module, &builder, &fpm, &ee);

    let fn_def = parse_fn_def("let tripleSum x y z = (x + y + z) * 3.0");
    compiler.compile_fn_def(&fn_def.as_ref()).unwrap();

    let fn_def = parse_fn_def("let tripleProduct x y z = (x * y * z) * 3.0");
    compiler.compile_fn_def(&fn_def.as_ref()).unwrap();

    let fn_def = parse_fn_def(
        "let tripleSumTripleProduct x y z = (tripleSum x y z) * (tripleProduct x y z)",
    );
    compiler.compile_fn_def(&fn_def.as_ref()).unwrap();

    let fn_def = parse_fn_def(
        "let addAllOtherTripled x y z =
            3 * (
                (tripleSum x y z) +
                (tripleProduct x y z) +
                (tripleSumTripleProduct x y z)
            )",
    );
    compiler.compile_fn_def(&fn_def.as_ref()).unwrap();

    unsafe {
        let triple_sum = compiler
            .execution_engine()
            .get_function::<F32FnType>("tripleSum")
            .unwrap();

        let triple_product = compiler
            .execution_engine()
            .get_function::<F32FnType>("tripleProduct")
            .unwrap();

        let triple_sum_triple_product = compiler
            .execution_engine()
            .get_function::<F32FnType>("tripleSumTripleProduct")
            .unwrap();

        let add_all_other_tripled = compiler
            .execution_engine()
            .get_function::<F32FnType>("addAllOtherTripled")
            .unwrap();

        let (x, y, z) = (1.1, 2.22, 3.333);

        assert_eq!(triple_sum.call(x, y, z), (x + y + z) * 3.0);
        assert_eq!(triple_product.call(x, y, z), (x * y * z) * 3.0);
        assert_eq!(
            triple_sum_triple_product.call(x, y, z),
            ((x + y + z) * 3.0) * ((x * y * z) * 3.0)
        );
        assert_eq!(
            triple_sum_triple_product.call(x, y, z),
            triple_sum.call(x, y, z) * triple_product.call(x, y, z)
        );

        assert_eq!(
            add_all_other_tripled.call(x, y, z),
            3.0 * (triple_sum.call(x, y, z)
                + triple_product.call(x, y, z)
                + triple_sum_triple_product.call(x, y, z))
        );
    }
}

#[test]
fn compile_0_arg_fn() {
    let context = Compiler::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compile_test");
    let fpm = Compiler::default_function_pass_manager(&module);
    let ee = Compiler::default_execution_engine(&module);
    let mut compiler = Compiler::new(&context, &module, &builder, &fpm, &ee);

    let func_def = parse_fn_def("let test1 = 100 * 420.69");
    compiler.compile_fn_def(func_def.as_ref()).unwrap();

    let func_def = parse_fn_def("let test2 = 10.0 * 42");
    compiler.compile_fn_def(func_def.as_ref()).unwrap();

    unsafe {
        let test1 = compiler
            .execution_engine()
            .get_function::<unsafe extern "C" fn() -> f32>("test1")
            .unwrap();

        assert_eq!(test1.call(), 100.0 * 420.69);
        assert_eq!(test1.call(), 100.0 * 420.69);
        assert_eq!(test1.call(), 100.0 * 420.69);

        let test2 = compiler
            .execution_engine()
            .get_function::<unsafe extern "C" fn() -> f32>("test2")
            .unwrap();

        assert_eq!(test2.call(), 420.0);
        assert_eq!(test2.call(), 420.0);
        assert_eq!(test2.call(), 420.0);
    }
}
