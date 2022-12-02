use super::{expression::Expression, walker::Walk, ASTDepth, VarIdentifier};
use std::{fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
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
    fn walk<V: super::visitor::Visitor<()>>(&self, v: &mut V) {
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

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct QueryGuards {
    guards: Vec<Rc<Expression>>,
}

impl QueryGuards {
    pub fn new<Guards: IntoIterator<Item = Rc<Expression>>>(guards: Guards) -> Self {
        QueryGuards {
            guards: guards.into_iter().collect(),
        }
    }

    pub fn prepend_guard(&mut self, arg: Rc<Expression>) {
        self.guards.insert(0, arg);
    }

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

impl Display for QueryGuards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for guard in self.guards.iter() {
            if !is_first {
                f.write_str("\n")?;
            }
            f.write_str("! ")?;
            guard.fmt(f)?;
            is_first = false;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct QueryBinding {
    ids: Vec<VarIdentifier>,
    val: Rc<Expression>,
    is_negated: bool,
}

impl QueryBinding {
    pub fn new(ids: Vec<VarIdentifier>, val: Rc<Expression>) -> Self {
        QueryBinding {
            ids,
            val,
            is_negated: false,
        }
    }

    pub fn new_negated(ids: Vec<VarIdentifier>, val: Rc<Expression>) -> Self {
        QueryBinding {
            ids,
            val,
            is_negated: true,
        }
    }

    pub fn is_negated(&self) -> bool {
        self.is_negated
    }

    pub fn ids(&self) -> &Vec<VarIdentifier> {
        &self.ids
    }

    pub fn val(&self) -> Rc<Expression> {
        Rc::clone(&self.val)
    }
}

impl Display for QueryBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("? ")?;

        let mut is_first = true;
        for id in self.ids.iter() {
            if !is_first {
                f.write_str(", ")?;
            }
            id.fmt(f)?;
            is_first = false;
        }

        if self.is_negated {
            f.write_str(" <!- ")?;
        } else {
            f.write_str(" <- ")?;
        }

        self.val.fmt(f)
    }
}
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct QueryBindings {
    bindings: Vec<QueryBinding>,
}

impl QueryBindings {
    pub fn new(bindings: Vec<QueryBinding>) -> Self {
        Self { bindings }
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        let mut is_first = true;
        for binding in self.bindings.iter() {
            if !is_first {
                f.write_str("\n")?;
            }
            binding.fmt(f)?;
            is_first = false;
        }
        Ok(())
    }
}

impl ASTDepth for QueryBindings {
    fn ast_depth(&self) -> usize {
        1 + self.bindings.iter().map(|b| b.ast_depth()).sum::<usize>()
    }
}
