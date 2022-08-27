use crate::Compiler;

#[test]
fn compile_arithmetic() {
    let compiler = Compiler::new();
    compiler.gen_self_calling_fn();
}
