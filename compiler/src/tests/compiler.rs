use crate::Compiler;

#[test]
fn compile_arithmetic() {
    let compiler = Compiler::new();
    let bool_ty = compiler.llvm.bool_type();
    println!("bool_ty: {:?}", bool_ty)
}
