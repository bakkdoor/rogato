use super::expression::Expression;
use std::fmt::Display;

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

    pub fn iter(&self) -> std::slice::Iter<Expression> {
        self.guards.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }
}

impl Display for QueryGuards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .guards
            .iter()
            .map(|g| format!("! {}", g))
            .fold(String::from(""), |acc, fmt| format!("{}\n{}", acc, fmt));

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryBinding {
    ids: Vec<String>,
    val: Box<Expression>,
}

impl QueryBinding {
    pub fn new(ids: Vec<String>, val: Box<Expression>) -> Self {
        QueryBinding { ids: ids, val: val }
    }

    pub fn value<'a>(&'a self) -> &'a Expression {
        &self.val
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
        f.write_fmt(format_args!("{} <- {}", fmt_str, self.val))
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

    pub fn iter(&self) -> std::slice::Iter<QueryBinding> {
        self.bindings.iter()
    }
}

impl Display for QueryBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .bindings
            .iter()
            .map(|binding| format!("? {}", binding))
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
