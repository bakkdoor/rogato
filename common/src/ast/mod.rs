use std::hash::Hash;
use std::rc::Rc;
use std::{cell::RefCell, fmt::Display};

use self::{
    expression::Expression, fn_def::FnDef, module_def::ModuleDef, type_expression::TypeDef,
};

pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod helpers;
pub mod if_else;
pub mod lambda;
pub mod let_expression;
pub mod literal;
pub mod module_def;
pub mod pattern;
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

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ASTNode {
    id: ASTId,
    ast: Rc<AST>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ExprNode {
    id: ASTId,
    expr: Rc<Expression>,
}

#[derive(Clone, Eq, Debug)]
pub enum AST {
    RootComment(String),
    FnDef(Rc<RefCell<FnDef>>),
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
            AST::RootComment(comment) => {
                f.write_str("//")?;
                comment.fmt(f)
            }
            AST::FnDef(fn_def) => fn_def.borrow().fmt(f),
            AST::ModuleDef(mod_def) => mod_def.fmt(f),
            AST::Use(id) => {
                f.write_str("use ")?;
                id.fmt(f)
            }
            AST::TypeDef(type_def) => type_def.fmt(f),
        }
    }
}

impl ASTDepth for AST {
    fn ast_depth(&self) -> usize {
        match self {
            AST::RootComment(_) => 1,
            AST::FnDef(fn_def) => fn_def.borrow().ast_depth(),
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

impl Hash for AST {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            AST::RootComment(c) => c.hash(state),
            AST::FnDef(fn_def) => fn_def.borrow().hash(state),
            AST::ModuleDef(mod_def) => mod_def.hash(state),
            AST::Use(id) => id.hash(state),
            AST::TypeDef(type_def) => type_def.hash(state),
        }
    }
}

impl PartialEq for AST {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (AST::RootComment(c1), AST::RootComment(c2)) => c1.eq(c2),
            (AST::FnDef(fn_def1), AST::FnDef(fn_def2)) => fn_def1.eq(fn_def2),
            (AST::ModuleDef(mod_def1), AST::ModuleDef(mod_def2)) => mod_def1.eq(mod_def2),
            (AST::Use(id1), AST::Use(id2)) => id1.eq(id2),
            (AST::TypeDef(type_def1), AST::TypeDef(type_def2)) => type_def1.eq(type_def2),
            (_, _) => false,
        }
    }
}
