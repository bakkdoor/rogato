use super::{expression::Expression, walker::Walk, ASTDepth};
use crate::rogato::{
    db::{val, val::Value},
    interpreter::{EvalContext, Evaluate},
};
use std::{fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Query {
    bindings: QueryBindings,
    guards: QueryGuards,
    production: Rc<Expression>,
}

impl Query {
    pub fn new(bindings: QueryBindings, guards: QueryGuards, production: Rc<Expression>) -> Self {
        Self {
            bindings,
            guards,
            production,
        }
    }

    pub fn bindings(&self) -> &QueryBindings {
        &self.bindings
    }

    pub fn guards(&self) -> &QueryGuards {
        &self.guards
    }

    pub fn production(&self) -> &Expression {
        &self.production
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

impl Evaluate<Value> for Query {
    fn evaluate(&self, context: &mut EvalContext) -> Value {
        match context.schedule_query(self) {
            Ok(val) => val,
            Err(e) => val::string(format!("error: {:?}", e)),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryGuards {
    guards: Vec<Rc<Expression>>,
}

impl QueryGuards {
    pub fn new(guards: Vec<Rc<Expression>>) -> Self {
        QueryGuards { guards }
    }

    #[allow(dead_code)]
    pub fn prepend_guard(&mut self, arg: Rc<Expression>) {
        self.guards.insert(0, arg);
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.guards.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<Expression>> {
        self.guards.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QueryGuardError {
    Unknown(String),
}

impl Evaluate<Vec<Result<Value, QueryGuardError>>> for QueryGuards {
    fn evaluate(&self, context: &mut EvalContext) -> Vec<Result<Value, QueryGuardError>> {
        let mut results = vec![];
        for guard in self.iter() {
            let value = guard.evaluate(context);
            if value.is_null() {
                results.push(Err(QueryGuardError::Unknown(format!(
                    "Not sure what went wrong but query guard failed: {:?}",
                    guard
                ))));
            } else {
                results.push(Ok(value))
            }
        }
        results
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
    val: Rc<Expression>,
    is_negated: bool,
}

impl QueryBinding {
    pub fn new(ids: Vec<String>, val: Rc<Expression>) -> Self {
        QueryBinding {
            ids,
            val,
            is_negated: false,
        }
    }

    pub fn new_negated(ids: Vec<String>, val: Rc<Expression>) -> Self {
        QueryBinding {
            ids,
            val,
            is_negated: true,
        }
    }

    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    pub fn value_expr(&self) -> Rc<Expression> {
        self.val.clone()
    }

    pub fn is_negated(&self) -> bool {
        self.is_negated
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
