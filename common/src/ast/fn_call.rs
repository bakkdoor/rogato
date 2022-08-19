use super::expression::Expression;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct FnCallArgs {
    args: Vec<Rc<Expression>>,
}

impl FnCallArgs {
    pub fn new(args: Vec<Rc<Expression>>) -> Self {
        FnCallArgs { args }
    }

    pub fn from_owned(args: Vec<Expression>) -> Self {
        FnCallArgs {
            args: args.iter().map(|a| Rc::new(a.clone())).collect(),
        }
    }

    pub fn prepend_arg(&mut self, arg: Rc<Expression>) {
        self.args.insert(0, arg);
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<Expression>> {
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
