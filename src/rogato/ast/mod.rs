use std::fmt::Display;

use indent_write::indentable::Indentable;

use self::{
    expression::{Expression, FnDefArgs},
    module_def::ModuleExports,
    type_expression::TypeExpression,
};

pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod module_def;
pub mod program;
pub mod query;
pub mod type_expression;
pub mod visitor;
pub mod walker;

pub type Identifier = String;

pub use program::Program;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AST {
    RootComment(String),
    FnDef(Identifier, FnDefArgs, Box<Expression>),
    ModuleDef(Identifier, ModuleExports),
    TypeDef(Identifier, Box<TypeExpression>),
}

impl Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AST::RootComment(comment) => f.write_fmt(format_args!("//{}", comment)),
            AST::FnDef(id, args, body) => f.write_fmt(format_args!(
                "let {}{} =\n{}",
                id,
                args,
                body.indented("    ")
            )),
            AST::ModuleDef(id, exports) => f.write_fmt(format_args!("module {} ({})", id, exports)),
            AST::TypeDef(id, type_expr) => {
                f.write_fmt(format_args!("type {} :: {}", id, type_expr))
            }
        }
    }
}
