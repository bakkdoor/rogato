use super::{expression::Expression, walker::Walk, ASTDepth};
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Query {
    bindings: QueryBindings,
    guards: QueryGuards,
    production: Box<Expression>,
}

impl Query {
    pub fn new(bindings: QueryBindings, guards: QueryGuards, production: Box<Expression>) -> Self {
        Self {
            bindings,
            guards,
            production,
        }
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.guards.is_empty() {
            f.write_fmt(format_args!("{}\n!> {}", self.bindings, self.production))
        } else {
            f.write_fmt(format_args!(
                "{}\n{}\n!> {}",
                self.bindings, self.guards, self.production
            ))
        }
    }
}

impl ASTDepth for Query {
    fn ast_depth(&self) -> usize {
        self.bindings.ast_depth()
            + self.guards.iter().map(|g| g.ast_depth()).sum::<usize>()
            + self.production.ast_depth()
            + 1
    }
}

impl Walk for Query {
    fn walk<V: super::visitor::Visitor>(&self, v: &mut V) {
        v.query(self);
        for binding in self.bindings.iter() {
            binding.val.walk(v);
        }
        for g in self.guards.iter() {
            g.walk(v);
        }
        self.production.walk(v);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryGuards {
    guards: Vec<Expression>,
}

impl QueryGuards {
    pub fn new(guards: Vec<Expression>) -> Self {
        QueryGuards { guards }
    }

    #[allow(dead_code)]
    pub fn prepend_guard(&mut self, arg: Expression) {
        self.guards.insert(0, arg);
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.guards.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Expression> {
        self.guards.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }
}

impl Display for QueryGuards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str =
            self.guards
                .iter()
                .map(|g| format!("! {}", g))
                .fold(String::from(""), |acc, fmt| {
                    if acc.is_empty() {
                        fmt
                    } else {
                        format!("{}\n{}", acc, fmt)
                    }
                });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryBinding {
    ids: Vec<String>,
    val: Box<Expression>,
    is_negated: bool,
}

impl QueryBinding {
    pub fn new(ids: Vec<String>, val: Box<Expression>) -> Self {
        QueryBinding {
            ids,
            val,
            is_negated: false,
        }
    }

    pub fn new_negated(ids: Vec<String>, val: Box<Expression>) -> Self {
        QueryBinding {
            ids,
            val,
            is_negated: true,
        }
    }
}

impl Display for QueryBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self.ids.iter().fold(String::from(""), |acc, id| {
            if acc.is_empty() {
                id.to_string()
            } else {
                format!("{}, {}", acc, id)
            }
        });

        if self.is_negated {
            f.write_fmt(format_args!("? {} <!- {}", fmt_str, self.val))
        } else {
            f.write_fmt(format_args!("? {} <- {}", fmt_str, self.val))
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryBindings {
    bindings: Vec<QueryBinding>,
}

impl QueryBindings {
    pub fn new(bindings: Vec<QueryBinding>) -> Self {
        Self { bindings }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn iter(&self) -> std::slice::Iter<QueryBinding> {
        self.bindings.iter()
    }
}

impl ASTDepth for QueryBinding {
    fn ast_depth(&self) -> usize {
        1 + self.val.ast_depth()
    }
}

impl Display for QueryBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .bindings
            .iter()
            .map(|binding| format!("{}", binding))
            .fold(String::from(""), |acc, fmt| {
                if acc.is_empty() {
                    fmt
                } else {
                    format!("{}\n{}", acc, fmt)
                }
            });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

impl ASTDepth for QueryBindings {
    fn ast_depth(&self) -> usize {
        1 + self.bindings.iter().map(|b| b.ast_depth()).sum::<usize>()
    }
}
