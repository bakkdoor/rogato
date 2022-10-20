use std::borrow::Borrow;

use crate::Compiler;

#[test]
fn compile_arithmetic() {
    let context = inkwell::context::Context::create();
    let module = context.borrow().create_module("test");
    let compiler = Compiler::new(&context, module);
    let bool_ty = compiler.context().bool_type();
    let void_ty = compiler.context().void_type();
    println!("bool_ty: {:?}", bool_ty);
    println!("void_ty: {:?}", void_ty);
}
