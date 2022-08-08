use std::fmt::Display;
use std::rc::Rc;

use self::{
    expression::Expression, fn_def::FnDef, module_def::ModuleDef, type_expression::TypeDef,
};

pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod helpers;
pub mod lambda;
pub mod let_expression;
pub mod literal;
pub mod module_def;
pub mod program;
pub mod query;
pub mod type_expression;
pub mod visitor;
pub mod walker;

use smol_str::SmolStr;
pub type Identifier = SmolStr;

impl Evaluate<ValueRef> for Identifier {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match context.lookup_var(self) {
            Some(val) => Ok(val),
            None => Err(EvalError::VarNotDefined(self.clone())),
        }
    }
}

pub use program::Program;

use super::db::{val, ValueRef};
use super::interpreter::{EvalContext, EvalError, Evaluate};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AST {
    RootComment(String),
    FnDef(Rc<FnDef>),
    ModuleDef(ModuleDef),
    Use(Identifier),
    TypeDef(TypeDef),
}

pub trait ASTDepth {
    fn ast_depth(&self) -> usize;
}

impl Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AST::RootComment(comment) => f.write_fmt(format_args!("//{}", comment)),
            AST::FnDef(fn_def) => f.write_fmt(format_args!("{}", fn_def)),
            AST::ModuleDef(mod_def) => f.write_fmt(format_args!("{}", mod_def)),
            AST::Use(id) => f.write_fmt(format_args!("use {}", id)),
            AST::TypeDef(type_def) => f.write_fmt(format_args!("{}", type_def)),
        }
    }
}

impl ASTDepth for AST {
    fn ast_depth(&self) -> usize {
        match self {
            AST::RootComment(_) => 1,
            AST::FnDef(fn_def) => fn_def.ast_depth(),
            AST::ModuleDef(mod_def) => mod_def.ast_depth(),
            AST::Use(_) => 1,
            AST::TypeDef(type_def) => type_def.ast_depth(),
        }
    }
}

impl Evaluate<ValueRef> for AST {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match self {
            AST::RootComment(_) => Ok(val::null()),
            AST::FnDef(fn_def) => fn_def.evaluate(context),
            AST::ModuleDef(mod_def) => mod_def.evaluate(context),
            AST::Use(_id) => Ok(val::null()),
            AST::TypeDef(type_def) => type_def.evaluate(context),
        }
    }
}
