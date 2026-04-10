use crate::Codegen;
use rogato_common::ast::fn_def::FnDef;
use rogato_parser::{parse_ast, ParserContext};

pub fn parse_fn_def(code: &str) -> std::rc::Rc<std::cell::RefCell<FnDef>> {
    let parser_ctx = ParserContext::new();
    let ast = parse_ast(code, &parser_ctx).unwrap();
    match ast.as_ref() {
        rogato_common::ast::AST::FnDef(f) => std::rc::Rc::clone(f),
        _ => panic!("Invalid AST node, expected FnDef"),
    }
}

#[test]
fn codegen_symbol_expressions() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let get_symbol = ^my_symbol");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();

    let function = unsafe {
        compiler
            .execution_engine
            .get_function::<unsafe extern "C" fn() -> *const i8>("get_symbol")
            .unwrap()
    };

    let result = unsafe { function.call() };
    assert!(!result.is_null());
}

#[test]
fn codegen_symbol_in_let_binding() {
    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module = context.create_module("compiler_test");
    let target_machine = Codegen::default_target_machine(&module);
    let ee = Codegen::default_execution_engine(&module);
    let mut compiler = Codegen::new(&context, &module, &builder, &target_machine, &ee);

    let func_def = parse_fn_def("let use_symbol x = let s = ^test in x");
    compiler.codegen_fn_def(&func_def.borrow()).unwrap();
}
