use crate::{
    eval::{native_fn::NativeFn, EvalContext, EvalError, Evaluate},
    util::indent,
    val,
    val::ValueRef,
};

use super::{expression::Expression, walker::Walk, ASTDepth, Identifier};
use std::{borrow::Borrow, fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnDef {
    id: Identifier,
    args: FnDefArgs,
    body: Rc<FnDefBody>,
}

impl FnDef {
    pub fn new<ID: Into<Identifier>>(id: ID, args: FnDefArgs, body: Rc<FnDefBody>) -> Rc<FnDef> {
        Rc::new(FnDef {
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

    pub fn body(&self) -> &FnDefBody {
        &self.body
    }
}

impl Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.body.borrow() {
            FnDefBody::NativeFn(_) => f.write_fmt(format_args!(
                "let {}{} =\n{}",
                self.id,
                self.args,
                indent("[NativeFn]")
            )),
            FnDefBody::RogatoFn(body_expr) => f.write_fmt(format_args!(
                "let {}{} =\n{}",
                self.id,
                self.args,
                indent(body_expr.to_owned())
            )),
        }
    }
}

impl ASTDepth for FnDef {
    fn ast_depth(&self) -> usize {
        1 + self.args.len() + self.body.ast_depth()
    }
}

impl Evaluate<ValueRef> for FnDef {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        context.define_fn(FnDef::new(
            self.id.clone(),
            self.args.clone(),
            self.body.clone(),
        ));
        Ok(val::object(vec![
            ("type", val::string("Fn")),
            ("name", val::string(self.id.to_string())),
            (
                "args",
                val::list(self.args.iter().map(val::string).collect::<Vec<ValueRef>>()),
            ),
        ]))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnDefArgs {
    args: Vec<Identifier>,
}

impl FnDefArgs {
    pub fn new(args: Vec<Identifier>) -> Self {
        FnDefArgs { args }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Identifier> {
        self.args.iter()
    }
}

impl Display for FnDefArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .iter()
            .map(|arg| arg.to_string())
            .fold(String::from(""), |acc, fmt| format!("{} {}", acc, fmt));

        f.write_fmt(format_args!("{}", fmt_str))
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

    pub fn call(
        &self,
        context: &mut EvalContext,
        args: &[ValueRef],
    ) -> Result<ValueRef, EvalError> {
        match self {
            FnDefBody::NativeFn(f) => f(context, args),
            FnDefBody::RogatoFn(expr) => expr.evaluate(context),
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
