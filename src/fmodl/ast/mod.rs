use std::fmt::Display;

use indent_write::indentable::Indentable;

use self::{
    expression::{Expression, FnDefArgs},
    module_def::ModuleExports,
};

pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod module_def;

pub type Identifier = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AST {
    FnDef(Identifier, FnDefArgs, Box<Expression>),
    ModuleDef(Identifier, ModuleExports),
}

impl Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AST::FnDef(id, args, body) => f.write_fmt(format_args!(
                "let {}{} =\n{}",
                id,
                args,
                body.indented("    ")
            )),
            AST::ModuleDef(id, exports) => f.write_fmt(format_args!("module {} ({})", id, exports)),
        }
    }
}
