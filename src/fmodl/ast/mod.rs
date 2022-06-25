use std::fmt::Display;

use self::expression::{Expression, FnDefArgs};

pub mod expression;
pub mod fn_call;
pub mod fn_def;

pub type Identifier = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AST {
    FnDef(Identifier, FnDefArgs, Box<Expression>),
}

impl Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AST::FnDef(id, args, body) => {
                f.write_fmt(format_args!("let {} {} = {}", id, args, body))
            }
        }
    }
}
