use super::expression::Expression;
use super::{ASTDepth, Identifier};
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct FnCall {
    pub id: Identifier,
    pub args: FnCallArgs,
}

impl FnCall {
    pub fn new(id: Identifier, args: FnCallArgs) -> Self {
        Self { id, args }
    }
}

impl ASTDepth for FnCall {
    fn ast_depth(&self) -> usize {
        1 + self.args.ast_depth()
    }
}

impl Display for FnCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        self.id.fmt(f)?;
        f.write_str(" ")?;
        self.args.fmt(f)?;
        f.write_str(")")
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct FnCallArgs<T: Clone = Expression> {
    args: Vec<Rc<T>>,
}

impl<T: Clone> FnCallArgs<T> {
    pub fn new<Args: IntoIterator<Item = Rc<T>>>(args: Args) -> Self {
        FnCallArgs {
            args: args.into_iter().collect(),
        }
    }

    pub fn from_owned(args: Vec<T>) -> Self {
        FnCallArgs {
            args: args.iter().map(|a| Rc::new(a.clone())).collect(),
        }
    }

    pub fn empty() -> Self {
        FnCallArgs { args: Vec::new() }
    }

    pub fn prepend_arg(&mut self, arg: Rc<T>) {
        self.args.insert(0, arg);
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<T>> {
        self.args.iter()
    }
}

impl Display for FnCallArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for arg in self.args.iter() {
            if !is_first {
                f.write_str(" ")?;
            }
            arg.fmt(f)?;
            is_first = false;
        }
        Ok(())
    }
}

impl<T: Clone + ASTDepth> ASTDepth for FnCallArgs<T> {
    fn ast_depth(&self) -> usize {
        self.iter().map(|a| a.ast_depth()).sum()
    }
}
