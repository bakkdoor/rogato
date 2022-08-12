pub mod parser;

pub use parser::{parse, parse_ast, parse_expr};
use rogato_common::ast::NodeFactory;

#[cfg(test)]
pub mod tests;

pub struct ParserContext {
    node_factory: NodeFactory,
}

impl ParserContext {
    pub fn new() -> ParserContext {
        ParserContext {
            node_factory: NodeFactory::new(),
        }
    }

    pub fn node_factory(&mut self) -> &NodeFactory {
        &self.node_factory
    }

    pub fn node_factory_mut(&mut self) -> &mut NodeFactory {
        &mut self.node_factory
    }
}

impl Default for ParserContext {
    fn default() -> Self {
        ParserContext::new()
    }
}
