use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::{util::indent, val::ValueRef};

use super::{
    expression::Expression, pattern::Pattern, walker::Walk, ASTDepth, Identifier, VarIdentifier,
};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct LambdaVariant {
    pub args: LambdaArgs<Rc<Pattern>>,
    pub body: Rc<Expression>,
}

impl LambdaVariant {
    pub fn new(args: LambdaArgs<Rc<Pattern>>, body: Rc<Expression>) -> LambdaVariant {
        LambdaVariant { args, body }
    }
    pub fn get_arg(&self, i: usize) -> Option<&Rc<Pattern>> {
        self.args.get(i)
    }

    pub fn arg_count(&self) -> usize {
        self.args.len()
    }
}

impl Display for LambdaVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.args.fmt(f)?;

        if self.ast_depth() > 5 {
            f.write_str(" ->\n")?;
            indent(&self.body).fmt(f)
        } else {
            f.write_str(" -> ")?;
            self.body.fmt(f)
        }
    }
}

impl ASTDepth for LambdaVariant {
    fn ast_depth(&self) -> usize {
        1 + self.args.len() + self.body.ast_depth()
    }
}

impl Walk for LambdaVariant {
    fn walk<V: super::visitor::Visitor<()>>(&self, v: &mut V) {
        v.lambda_variant(self);
        for arg in self.args.iter() {
            arg.walk(v);
        }
        self.body.walk(v);
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct LambdaArgs<A: Display + ASTDepth> {
    args: Vec<A>,
}

impl<A: Display + ASTDepth> LambdaArgs<A> {
    pub fn new(args: Vec<A>) -> LambdaArgs<A> {
        LambdaArgs { args }
    }

    pub fn empty() -> LambdaArgs<A> {
        LambdaArgs { args: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<A> {
        self.args.iter()
    }

    pub fn get(&self, idx: usize) -> Option<&A> {
        self.args.get(idx)
    }
}

impl<A: Display + ASTDepth> Display for LambdaArgs<A> {
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

impl<A: Display + ASTDepth> ASTDepth for LambdaArgs<A> {
    fn ast_depth(&self) -> usize {
        self.args.iter().map(|a| a.ast_depth()).sum::<usize>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lambda {
    variants: Vec<Rc<LambdaVariant>>,
}

impl Lambda {
    pub fn new(variants: Vec<Rc<LambdaVariant>>) -> Self {
        Self { variants }
    }

    pub fn variants_iter(&self) -> std::slice::Iter<Rc<LambdaVariant>> {
        self.variants.iter()
    }

    pub fn max_arg_count(&self) -> usize {
        let mut max = 0;
        for v in self.variants_iter() {
            if v.arg_count() > max {
                max = v.arg_count();
            }
        }
        max
    }
}

impl ASTDepth for Lambda {
    fn ast_depth(&self) -> usize {
        self.variants.iter().map(|v| v.ast_depth()).sum()
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(\n")?;
        let mut is_first = true;
        for lambda in self.variants.iter() {
            if !is_first {
                f.write_str(",\n")?;
            }
            indent(&lambda.args).fmt(f)?;

            if self.ast_depth() > 8 {
                f.write_str(" ->\n")?;
                indent(&indent(&lambda.body)).fmt(f)?;
            } else {
                f.write_str(" -> ")?;
                lambda.body.fmt(f)?;
            }
            is_first = false;
        }
        f.write_str("\n)")
    }
}

impl Walk for Lambda {
    fn walk<V: super::visitor::Visitor<()>>(&self, v: &mut V) {
        v.lambda(self);
        for lambda_variant in self.variants_iter() {
            lambda_variant.walk(v)
        }
    }
}

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum LambdaClosureEvalError {
    #[error("Unknown error in LambdaClosure {0}: {1}")]
    Unknown(Identifier, String),

    #[error("Evaluation failed for LambdaClosure with: {0}")]
    EvaluationFailed(Rc<LambdaVariant>, String),

    #[error("Lambda arity mismatch: Expected: {0} but got: {1}")]
    LambdaArityMismatch(usize, usize),

    #[error("LambdaVariant arguments did not match argument patterns: {0} / {1:?}")]
    LambdaVariantArgumentsMismatch(Rc<LambdaVariant>, Vec<ValueRef>),

    #[error("LambdaVariant argument pattern matching failed: {0} / {1} / {2}")]
    LambdaVariantArgumentMismatch(Rc<LambdaVariant>, Rc<Pattern>, ValueRef),

    #[error("Lambda arguments did not match argument patterns: {0} / {1:?}")]
    LambdaArgumentsMismatch(Lambda, Vec<ValueRef>),

    #[error("Lambda argument pattern matching failed: {0} / {1} / {2}")]
    LambdaArgumentMismatch(Lambda, Rc<Pattern>, ValueRef),
}

pub trait LambdaClosureContext {
    fn hash_id(&self) -> String;
    fn lookup_var(&self, id: &VarIdentifier) -> Option<ValueRef>;
    fn define_var(&mut self, id: &VarIdentifier, val: ValueRef);
    fn with_child_env(&self) -> Box<dyn LambdaClosureContext>;

    fn evaluate_lambda_call(
        &mut self,
        lambda: &Lambda,
        args: &[ValueRef],
    ) -> Result<ValueRef, LambdaClosureEvalError>;
}

impl PartialEq for dyn LambdaClosureContext {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for dyn LambdaClosureContext {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Hash for dyn LambdaClosureContext {
    fn hash<H: Hasher>(&self, h: &mut H) {
        Hash::hash(&self.hash_id(), h);
    }
}

impl std::fmt::Debug for dyn LambdaClosureContext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("LambdaClosureContext").finish()
    }
}
