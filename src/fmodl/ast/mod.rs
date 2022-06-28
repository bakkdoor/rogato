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
pub struct Program {
    nodes: Vec<Box<AST>>,
}

impl Program {
    pub fn new(nodes: Vec<AST>) -> Self {
        Self::from(Vec::from_iter(nodes.iter().map(|d| Box::new(d.clone()))))
    }

    pub fn from(nodes: Vec<Box<AST>>) -> Self {
        Program { nodes: nodes }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str =
            self.nodes
                .iter()
                .map(|ast| format!("{}", ast))
                .fold(String::from(""), |acc, fmt| {
                    if acc == "" {
                        fmt
                    } else {
                        format!("{}\n\n{}", acc, fmt)
                    }
                });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AST {
    RootComment(String),
    FnDef(Identifier, FnDefArgs, Box<Expression>),
    ModuleDef(Identifier, ModuleExports),
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
        }
    }
}
