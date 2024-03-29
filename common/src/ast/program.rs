use super::AST;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program {
    nodes: Vec<Rc<AST>>,
}

impl Program {
    pub fn new(nodes: Vec<Rc<AST>>) -> Self {
        Program { nodes }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<AST>> {
        self.nodes.iter()
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for ast in self.iter() {
            if !is_first {
                f.write_str("\n\n")?;
            }
            ast.fmt(f)?;
            is_first = false;
        }
        Ok(())
    }
}

impl FromIterator<Rc<AST>> for Program {
    fn from_iter<T: IntoIterator<Item = Rc<AST>>>(iter: T) -> Self {
        let mut nodes = vec![];
        for val in iter.into_iter() {
            nodes.push(val)
        }
        Program { nodes }
    }
}
