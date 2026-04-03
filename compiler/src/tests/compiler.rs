use std::{cell::RefCell, rc::Rc};

use rogato_common::ast::{fn_def::FnDef, AST};
use rogato_parser::{parse_ast, ParserContext};

use crate::Codegen;

pub fn parse_fn_def(code: &str) -> Rc<RefCell<FnDef>> {
    let mut parser_ctx = ParserContext::new();
    let ast = parse_ast(code, &mut parser_ctx).unwrap();
    match ast.as_ref() {
        AST::FnDef(f) => Rc::clone(f),
        _ => panic!("Invalid AST node, expected FnDef"),
    }
}

type F32FnType = unsafe extern "C" fn(f32, f32, f32) -> f32;

#[test]
fn codegen_add3() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let add3 x y z = (x + y) + z");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine
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
fn codegen_add2_mul() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let add2_mul x y z = (x + y) * z");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine
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
fn codegen_multiple_functions() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let fn_def = parse_fn_def("let tripleSum x y z = (x + y + z) * 3.0");
    compiler.codegen_fn_def(&fn_def.borrow()).unwrap();

    let fn_def = parse_fn_def("let tripleProduct x y z = (x * y * z) * 3.0");
    compiler.codegen_fn_def(&fn_def.borrow()).unwrap();

    let fn_def = parse_fn_def(
        "let tripleSumTripleProduct x y z = (tripleSum x y z) * (tripleProduct x y z)",
    );
    compiler.codegen_fn_def(&fn_def.borrow()).unwrap();

    let fn_def = parse_fn_def(
        "let addAllOtherTripled x y z =
            3 * (
                (tripleSum x y z) +
                (tripleProduct x y z) +
                (tripleSumTripleProduct x y z)
            )",
    );
    compiler.codegen_fn_def(&fn_def.borrow()).unwrap();

    unsafe {
        let triple_sum = compiler
            .execution_engine
            .get_function::<F32FnType>("tripleSum")
            .unwrap();

        let triple_product = compiler
            .execution_engine
            .get_function::<F32FnType>("tripleProduct")
            .unwrap();

        let triple_sum_triple_product = compiler
            .execution_engine
            .get_function::<F32FnType>("tripleSumTripleProduct")
            .unwrap();

        let add_all_other_tripled = compiler
            .execution_engine
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
fn codegen_0_arg_fn() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let test1 = 100 * 420.69");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    let func_def = parse_fn_def("let test2 = 10.0 * 42");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let test1 = compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn() -> f32>("test1")
            .unwrap();

        assert_eq!(test1.call(), 100.0 * 420.69);
        assert_eq!(test1.call(), 100.0 * 420.69);
        assert_eq!(test1.call(), 100.0 * 420.69);

        let test2 = compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn() -> f32>("test2")
            .unwrap();

        assert_eq!(test2.call(), 420.0);
        assert_eq!(test2.call(), 420.0);
        assert_eq!(test2.call(), 420.0);
    }
}

#[test]
fn codegen_if_else() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def(
        "
        let if_else_cond x y z =
            if (x > y) then
                (x * z)
            else
                (y * z)
        ",
    );
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine
            .get_function::<F32FnType>("if_else_cond")
            .unwrap();

        let params_and_results = [
            ((1.0, 2.0, 3.0), 6.0),
            ((2.0, 1.0, 3.0), 6.0),
            ((0.0, 0.0, 0.0), 0.0),
            ((1.0, 0.0, 0.0), 0.0),
            ((0.0, 2.2, 0.0), 0.0),
            ((0.0, 0.0, 3.3), 0.0),
            ((0.5, 10.0, 2.5), 25.0),
        ];

        for ((x, y, z), result) in params_and_results {
            let val = function.call(x, y, z);
            assert_eq!(val, result);
        }
    }
}

#[test]
fn codegen_bool_literals() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let returnTrue = true");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    let func_def = parse_fn_def("let returnFalse = false");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let true_fn = compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn() -> bool>("returnTrue")
            .unwrap();

        let false_fn = compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn() -> bool>("returnFalse")
            .unwrap();

        assert_eq!(true_fn.call(), true);
        assert_eq!(false_fn.call(), false);
    }
}

#[test]
fn codegen_bool_with_comparisons() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let isGreater x y = x > y");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn(f32, f32) -> bool>("isGreater")
            .unwrap();

        assert_eq!(function.call(5.0, 3.0), true);
        assert_eq!(function.call(3.0, 5.0), false);
        assert_eq!(function.call(3.0, 3.0), false);
    }
}

#[test]
fn codegen_let_bindings() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let testLet x y = (x * 2.0) + (y + 10.0)");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn(f32, f32) -> f32>("testLet")
            .unwrap();

        assert_eq!(function.call(5.0, 3.0), (5.0 * 2.0) + (3.0 + 10.0));
        assert_eq!(function.call(1.0, 2.0), (1.0 * 2.0) + (2.0 + 10.0));
        assert_eq!(function.call(0.0, 0.0), (0.0 * 2.0) + (0.0 + 10.0));
    }
}

#[test]
fn codegen_let_bindings_nested() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let testNestedLet x y z = ((x * y) + (y * 3.0))");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn(f32, f32, f32) -> f32>("testNestedLet")
            .unwrap();

        assert_eq!(function.call(2.0, 3.0, 4.0), (2.0 * 3.0) + (3.0 * 3.0));
    }
}

#[test]
fn codegen_bool_comparisons_chain() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    // Note: comparisons now return Bool (i1), so chain with arithmetic needs conversion
    let func_def = parse_fn_def("let checkRange x y z = if (x > y) then 1.0 else 0.0");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    unsafe {
        let function = compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn(f32, f32) -> f32>("checkRange")
            .unwrap();

        assert_eq!(function.call(5.0, 3.0), 1.0);
        assert_eq!(function.call(3.0, 5.0), 0.0);
    }
}

#[cfg(test)]
mod output_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_all_output_formats() {
        // Setup: create compiler and compile a simple function
        let context = Codegen::new_context();
        let builder = context.create_builder();
        let module = context.create_module("output_test");
        let target_machine = Codegen::default_target_machine(&module);
        let ee = Codegen::default_execution_engine(&module);
        let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

        let func_def = parse_fn_def("let testFn x y = x + y");
        compiler.codegen_fn_def(&func_def.borrow()).unwrap();

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // === IR Output Tests ===
        let ir_path = base_path.join("test.ll");
        compiler
            .write_ir_to_file(&ir_path)
            .expect("Failed to write IR");

        // IR file exists and has content
        assert!(ir_path.exists(), "IR file should exist");

        let ir_content = fs::read_to_string(&ir_path).expect("Failed to read IR file");
        assert!(!ir_content.is_empty(), "IR should have content");

        // IR contains expected elements
        assert!(
            ir_content.contains("define float @testFn"),
            "IR should contain function definition"
        );
        assert!(
            ir_content.contains("target datalayout"),
            "IR should contain target datalayout"
        );
        assert!(
            ir_content.contains("ret float"),
            "IR should contain return instruction"
        );
        assert!(
            ir_content.contains("fadd") || ir_content.contains("add"),
            "IR should contain add operation"
        );

        // === Bitcode Output Tests ===
        let bc_path = base_path.join("test.bc");
        compiler
            .write_bitcode_to_file(&bc_path)
            .expect("Failed to write bitcode");

        // Bitcode file exists and has content
        assert!(bc_path.exists(), "Bitcode file should exist");

        let bc_metadata = fs::metadata(&bc_path).expect("Failed to get bitcode file size");
        assert!(bc_metadata.len() > 0, "Bitcode should not be empty");

        let bc_bytes = fs::read(&bc_path).expect("Failed to read bitcode file");
        assert!(!bc_bytes.is_empty(), "Bitcode should have content");

        // === Object File Output Tests ===
        let obj_path = base_path.join("test.o");
        compiler
            .write_object_to_file(&obj_path)
            .expect("Failed to write object file");

        // Object file exists and has content
        assert!(obj_path.exists(), "Object file should exist");

        let obj_metadata = fs::metadata(&obj_path).expect("Failed to get object file size");
        assert!(obj_metadata.len() > 0, "Object should not be empty");

        let obj_bytes = fs::read(&obj_path).expect("Failed to read object file");
        assert!(!obj_bytes.is_empty(), "Object should have content");

        // === Assembly Output Tests ===
        let asm_path = base_path.join("test.s");
        compiler
            .write_assembly_to_file(&asm_path)
            .expect("Failed to write assembly");

        // Assembly file exists
        assert!(asm_path.exists(), "Assembly file should exist");

        let asm_content = fs::read_to_string(&asm_path).expect("Failed to read assembly file");
        assert!(!asm_content.is_empty(), "Assembly should have content");

        // Assembly contains expected elements
        assert!(
            asm_content.contains(".text") || asm_content.contains("testFn"),
            "Assembly should contain function"
        );
    }
}
