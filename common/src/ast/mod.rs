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

pub use program::Program;

pub type ASTId = usize;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NodeFactory {
    current_id: ASTId,
}

impl NodeFactory {
    pub fn new() -> NodeFactory {
        NodeFactory { current_id: 0 }
    }

    pub fn next_id(&mut self) -> ASTId {
        self.current_id += 1;
        self.current_id
    }

    pub fn ast_node(&mut self, ast: Rc<AST>) -> ASTNode {
        let id = self.next_id();
        ASTNode { id, ast }
    }

    pub fn expr_node(&mut self, expr: Rc<Expression>) -> ExprNode {
        let id = self.next_id();
        ExprNode { id, expr }
    }
}

impl Default for NodeFactory {
    fn default() -> Self {
        NodeFactory::new()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ASTNode {
    id: ASTId,
    ast: Rc<AST>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprNode {
    id: ASTId,
    expr: Rc<Expression>,
}

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

impl ASTDepth for Identifier {
    fn ast_depth(&self) -> usize {
        1
    }
}
