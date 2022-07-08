use std::fmt::Display;

use super::AST;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program {
    nodes: Vec<Box<AST>>,
}

impl Program {
    pub fn new(nodes: Vec<AST>) -> Self {
        Self::from(Vec::from_iter(nodes.iter().map(|d| Box::new(d.clone()))))
    }

    pub fn from(nodes: Vec<Box<AST>>) -> Self {
        Program { nodes: nodes }
    }

    pub fn iter(&self) -> std::slice::Iter<Box<AST>> {
        self.nodes.iter()
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str =
            self.iter()
                .map(|ast| format!("{}", ast))
                .fold(String::from(""), |acc, fmt| {
                    if acc == "" {
                        fmt
                    } else {
                        format!("{}\n\n{}", acc, fmt)
                    }
                });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
