use super::pattern::Pattern;
use super::{expression::Expression, walker::Walk, ASTDepth, Identifier};
use crate::{native_fn::NativeFn, util::indent};
use std::hash::{Hash, Hasher};
use std::{borrow::Borrow, fmt::Display, rc::Rc};

#[derive(Clone, Debug, Eq)]
pub struct FnDef {
    is_inline: bool,
    id: Identifier,
    args: FnDefArgs,
    body: Rc<FnDefBody>,
}

impl PartialEq for FnDef {
    fn eq(&self, other: &Self) -> bool {
        self.is_inline.eq(&other.is_inline)
            && self.id.eq(&other.id)
            && self.args.eq(&other.args)
            && self.body.eq(&other.body)
    }
}

impl Hash for FnDef {
    fn hash<H: Hasher>(&self, h: &mut H) {
        Hash::hash(&self.id, h);
        Hash::hash(&self.is_inline, h);
        Hash::hash(&self.args, h);
    }
}

impl FnDef {
    pub fn new<ID: Into<Identifier>>(id: ID, args: FnDefArgs, body: Rc<FnDefBody>) -> Rc<FnDef> {
        Rc::new(FnDef {
            is_inline: false,
            id: id.into(),
            args,
            body,
        })
    }

    pub fn new_inline<ID: Into<Identifier>>(
        id: ID,
        args: FnDefArgs,
        body: Rc<FnDefBody>,
    ) -> Rc<FnDef> {
        Rc::new(FnDef {
            is_inline: true,
            id: id.into(),
            args,
            body,
        })
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn args(&self) -> &FnDefArgs {
        &self.args
    }

    pub fn body(&self) -> Rc<FnDefBody> {
        Rc::clone(&self.body)
    }
}

impl Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.is_inline {
            f.write_str("let ")?;
        }

        self.id.fmt(f)?;
        f.write_str(" ")?;
        self.args.fmt(f)?;
        f.write_str(" =\n")?;

        match self.body.borrow() {
            FnDefBody::NativeFn(_) => indent(&"[NativeFn]").fmt(f),
            FnDefBody::RogatoFn(body_expr) => indent(body_expr).fmt(f),
        }
    }
}

impl ASTDepth for FnDef {
    fn ast_depth(&self) -> usize {
        1 + self.args.len() + self.body.ast_depth()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct FnDefArgs {
    args: Vec<Rc<Pattern>>,
}

impl FnDefArgs {
    pub fn new(args: Vec<Rc<Pattern>>) -> Self {
        FnDefArgs { args }
    }

    pub fn required_args(&self) -> usize {
        let optional_args = self
            .args
            .iter()
            .filter(|a| Self::is_optional_arg(a))
            .count();
        self.len() - optional_args
    }

    pub fn is_optional_arg(p: &Pattern) -> bool {
        match p {
            Pattern::Var(v) => v.starts_with('?'),
            _ => false, // TODO: ???
        }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<Pattern>> {
        self.args.iter()
    }
}

impl Display for FnDefArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = false;
        for arg in self.iter() {
            if is_first {
                f.write_str(" ")?;
            }
            arg.fmt(f)?;
            is_first = true;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum FnDefBody {
    NativeFn(NativeFn),
    RogatoFn(Rc<Expression>),
}

impl FnDefBody {
    pub fn native(f: NativeFn) -> FnDefBody {
        FnDefBody::NativeFn(f)
    }

    pub fn rogato(expr: Rc<Expression>) -> FnDefBody {
        FnDefBody::RogatoFn(expr)
    }
}

impl PartialEq for FnDefBody {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FnDefBody::RogatoFn(a), FnDefBody::RogatoFn(b)) => a.eq(b),
            (_, _) => false,
        }
    }
}

impl Eq for FnDefBody {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::fmt::Debug for FnDefBody {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            FnDefBody::NativeFn(_) => f
                .debug_struct("NativeFn")
                .field("func", &"[NativeFn]")
                .finish(),
            FnDefBody::RogatoFn(expr) => f.debug_struct("RogatoFn").field("expr", &expr).finish(),
        }
    }
}

impl ASTDepth for FnDefBody {
    fn ast_depth(&self) -> usize {
        match self {
            FnDefBody::RogatoFn(expr) => expr.ast_depth(),
            FnDefBody::NativeFn(_) => 1,
        }
    }
}

impl Walk for FnDefBody {
    fn walk<V: super::visitor::Visitor<()>>(&self, v: &mut V) {
        match self {
            FnDefBody::RogatoFn(expr) => expr.walk(v),
            FnDefBody::NativeFn(_) => {}
        }
    }
}
