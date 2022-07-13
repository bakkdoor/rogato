use crate::rogato::util::indent;

use super::{expression::Expression, ASTDepth, Identifier};
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnDef {
    id: Identifier,
    args: FnDefArgs,
    body: Box<Expression>,
}

impl FnDef {
    pub fn new(id: Identifier, args: FnDefArgs, body: Box<Expression>) -> FnDef {
        FnDef { id, args, body }
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }
}

impl Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "let {}{} =\n{}",
            self.id,
            self.args,
            indent(self.body.to_owned())
        ))
    }
}

impl ASTDepth for FnDef {
    fn ast_depth(&self) -> usize {
        1 + self.args.len() + self.body.ast_depth()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnDefArgs {
    args: Vec<Identifier>,
}

impl FnDefArgs {
    pub fn new(args: Vec<Identifier>) -> Self {
        FnDefArgs { args }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.args.len()
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::slice::Iter<String> {
        self.args.iter()
    }
}

impl Display for FnDefArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .args
            .iter()
            .map(|arg| arg.to_string())
            .fold(String::from(""), |acc, fmt| format!("{} {}", acc, fmt));

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
