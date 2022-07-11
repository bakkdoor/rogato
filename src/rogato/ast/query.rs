use super::{expression::Expression, walker::Walk};
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Query {
    bindings: Box<QueryBindings>,
    guards: Box<QueryGuards>,
    production: Box<Expression>,
}

impl Query {
    pub fn new(
        bindings: Box<QueryBindings>,
        guards: Box<QueryGuards>,
        production: Box<Expression>,
    ) -> Self {
        Self {
            bindings: bindings,
            guards: guards,
            production: production,
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
    guards: Box<Vec<Expression>>,
}

impl QueryGuards {
    pub fn new(guards: Vec<Expression>) -> Self {
        QueryGuards {
            guards: Box::new(guards),
        }
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
                    if acc == "" {
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
            ids: ids,
            val: val,
            is_negated: false,
        }
    }

    pub fn new_negated(ids: Vec<String>, val: Box<Expression>) -> Self {
        QueryBinding {
            ids: ids,
            val: val,
            is_negated: true,
        }
    }
}

impl Display for QueryBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str =
            self.ids
                .iter()
                .map(|id| format!("{}", id))
                .fold(String::from(""), |acc, fmt| {
                    if acc == "" {
                        fmt
                    } else {
                        format!("{}, {}", acc, fmt)
                    }
                });

        if self.is_negated {
            f.write_fmt(format_args!("?! {} <- {}", fmt_str, self.val))
        } else {
            f.write_fmt(format_args!("? {} <- {}", fmt_str, self.val))
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryBindings {
    bindings: Box<Vec<QueryBinding>>,
}

impl QueryBindings {
    pub fn new(bindings: Box<Vec<QueryBinding>>) -> Self {
        Self { bindings: bindings }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn iter(&self) -> std::slice::Iter<QueryBinding> {
        self.bindings.iter()
    }
}

impl Display for QueryBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .bindings
            .iter()
            .map(|binding| format!("{}", binding))
            .fold(String::from(""), |acc, fmt| {
                if acc == "" {
                    fmt
                } else {
                    format!("{}\n{}", acc, fmt)
                }
            });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
