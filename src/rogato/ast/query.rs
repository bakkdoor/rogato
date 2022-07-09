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
