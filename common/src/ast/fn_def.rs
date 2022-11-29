use super::pattern::Pattern;
use super::{expression::Expression, walker::Walk, ASTDepth, Identifier};
use crate::{native_fn::NativeFn, util::indent};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::{fmt::Display, rc::Rc};

#[derive(Clone, Debug, Eq)]
pub struct FnDef {
    pub is_inline: bool,
    pub id: Identifier,
    pub variants: FnDefVariants,
    is_tail_recursive: bool,
}

impl PartialEq for FnDef {
    fn eq(&self, other: &Self) -> bool {
        self.is_inline.eq(&other.is_inline)
            && self.id.eq(&other.id)
            && self.variants.eq(&other.variants)
            && self.is_tail_recursive.eq(&other.is_tail_recursive)
    }
}

impl Hash for FnDef {
    fn hash<H: Hasher>(&self, h: &mut H) {
        Hash::hash(&self.id, h);
        Hash::hash(&self.is_inline, h);
        Hash::hash(&self.variants, h);
        Hash::hash(&self.is_tail_recursive, h);
    }
}

impl FnDef {
    pub fn new<ID: Into<Identifier>>(
        id: ID,
        args: FnDefArgs,
        body: Rc<FnDefBody>,
    ) -> Rc<RefCell<FnDef>> {
        let id = id.into();
        let is_tail_recursive = body.is_tail_recursive(&id);
        Rc::new(RefCell::new(FnDef {
            is_inline: false,
            id,
            variants: FnDefVariants::new([(args, body)]),
            is_tail_recursive,
        }))
    }

    pub fn new_inline<ID: Into<Identifier>>(
        id: ID,
        args: FnDefArgs,
        body: Rc<FnDefBody>,
    ) -> Rc<RefCell<FnDef>> {
        let id = id.into();
        let is_tail_recursive = body.is_tail_recursive(&id);
        Rc::new(RefCell::new(FnDef {
            is_inline: true,
            id,
            variants: FnDefVariants::new([(args, body)]),
            is_tail_recursive,
        }))
    }

    pub fn add_variant(&mut self, args: FnDefArgs, body: Rc<FnDefBody>) {
        self.is_tail_recursive = self.is_tail_recursive || body.is_tail_recursive(&self.id);
        self.variants.add(args, body);
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn required_args(&self) -> usize {
        self.variants
            .iter()
            .map(|(args, _)| args.required_args())
            .min()
            .unwrap_or_default()
    }

    pub fn get_variant(&self, index: usize) -> Option<&FnDefVariant> {
        self.variants.get_variant(index)
    }

    pub fn is_tail_recursive(&self) -> bool {
        self.is_tail_recursive
    }
}

impl Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (args, body) in self.variants.iter() {
            if !self.is_inline {
                f.write_str("let ")?;
            }

            self.id.fmt(f)?;
            f.write_str(" ")?;
            args.fmt(f)?;
            f.write_str(" =\n")?;

            match &**body {
                FnDefBody::NativeFn(_) => indent(&"[NativeFn]").fmt(f)?,
                FnDefBody::RogatoFn(body_expr) => indent(&body_expr).fmt(f)?,
            }
        }

        Ok(())
    }
}

impl ASTDepth for FnDef {
    fn ast_depth(&self) -> usize {
        1 + self.variants.ast_depth()
    }
}

pub type FnDefVariant = (FnDefArgs, Rc<FnDefBody>);

impl ASTDepth for FnDefVariant {
    fn ast_depth(&self) -> usize {
        self.0.ast_depth() + self.1.ast_depth()
    }
}

#[derive(Clone, Debug, Eq)]
pub struct FnDefVariants {
    variants: Vec<FnDefVariant>,
}

impl FnDefVariants {
    pub fn new<V: Into<Vec<FnDefVariant>>>(variants: V) -> Self {
        Self {
            variants: variants.into(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<FnDefVariant> {
        self.variants.iter()
    }

    pub fn add(&mut self, args: FnDefArgs, body: Rc<FnDefBody>) {
        self.variants.push((args, body))
    }

    pub fn get_variant(&self, index: usize) -> Option<&FnDefVariant> {
        self.variants.get(index)
    }
}

impl ASTDepth for FnDefVariants {
    fn ast_depth(&self) -> usize {
        self.variants
            .iter()
            .map(|v| v.0.len() + v.1.ast_depth())
            .sum::<usize>()
    }
}

impl Hash for FnDefVariants {
    fn hash<H: Hasher>(&self, h: &mut H) {
        for v in self.variants.iter() {
            Hash::hash(&v.0, h);
        }
    }
}

impl PartialEq for FnDefVariants {
    fn eq(&self, other: &Self) -> bool {
        self.variants.eq(&other.variants)
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

impl ASTDepth for FnDefArgs {
    fn ast_depth(&self) -> usize {
        self.iter().map(|a| a.ast_depth()).sum::<usize>()
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

    pub fn is_tail_recursive(&self, id: &Identifier) -> bool {
        match self {
            Self::RogatoFn(body) => match body.deref() {
                Expression::FnCall(fn_call) => fn_call.id == *id,
                _ => false,
            },
            _ => false,
        }
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
