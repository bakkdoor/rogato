pub mod parser;
pub use parser::{parse, parse_ast, parse_expr};

use rogato_common::ast::NodeFactory;

pub use rogato_common::ast::SourceLocation;

pub type ParseError = peg::error::ParseError<peg::str::LineCol>;

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

    pub fn next_node_id(&mut self) -> usize {
        self.node_factory_mut().next_id().0
    }

    pub fn node_location(&self, file: &str, line: usize, col: usize) -> SourceLocation {
        SourceLocation {
            file: file.to_string(),
            line,
            column: col,
        }
    }
}

impl Default for ParserContext {
    fn default() -> Self {
        ParserContext::new()
    }
}
