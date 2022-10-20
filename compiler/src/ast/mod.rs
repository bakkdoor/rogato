use crate::{Compile, CompileResult, Compiler};
use inkwell::values::FloatValue;
use rogato_common::ast::AST;

pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod if_else;
pub mod lambda;
pub mod let_expression;
pub mod literal;
pub mod module_def;
pub mod program;
pub mod query;
pub mod type_def;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for AST {
    fn compile(&self, compiler: &'ctx mut Compiler<'ctx>) -> CompileResult<'ctx, FloatValue<'ctx>> {
        match self {
            AST::RootComment(_) => todo!(),
            AST::FnDef(fn_def) => fn_def.compile(compiler),
            AST::ModuleDef(mod_def) => mod_def.compile(compiler),
            AST::Use(_) => todo!(),
            AST::TypeDef(type_def) => type_def.compile(compiler),
        }
    }
}
