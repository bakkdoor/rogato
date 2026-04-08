pub use super::fn_call::{FnCall, FnCallArgs};
use super::fn_def::FnDef;
pub use super::fn_def::FnDefArgs;
pub use super::if_else::IfElse;
pub use super::lambda::{Lambda, LambdaArgs, LambdaVariant};
pub use super::let_expression::{LetBindings, LetExpression};
pub use super::literal::*;
pub use super::query::{Query, QueryBinding, QueryBindings, QueryGuards};
use super::{ASTDepth, Identifier, VarIdentifier, AST};
use crate::span::Span;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

/// The kind of an expression node (the actual variant data).
#[derive(Clone, Eq, Debug)]
pub enum ExprKind {
    Commented(String, Rc<Expression>),
    Lit(Literal),
    FnCall(FnCall),
    OpCall(Identifier, Rc<Expression>, Rc<Expression>),
    Var(VarIdentifier),
    ConstOrTypeRef(Identifier),
    DBTypeRef(Identifier),
    PropFnRef(Identifier),
    EdgeProp(Rc<Expression>, Identifier),
    IfElse(IfElse),
    Let(LetExpression),
    Lambda(Rc<Lambda>),
    Query(Query),
    Symbol(Identifier),
    Quoted(Rc<Expression>),
    QuotedAST(Rc<AST>),
    Unquoted(Rc<Expression>),
    UnquotedAST(Rc<AST>),
    InlineFnDef(Rc<RefCell<FnDef>>),
}

/// An expression node with an optional source span.
#[derive(Clone, Eq, Debug)]
pub struct Expression {
    pub kind: ExprKind,
    pub span: Option<Span>,
}

impl Expression {
    /// Create an expression with a known source span.
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Expression {
            kind,
            span: Some(span),
        }
    }

    /// Create an expression without source span information.
    pub fn unspanned(kind: ExprKind) -> Self {
        Expression { kind, span: None }
    }

    /// Create an expression with an optional span.
    pub fn with_span(kind: ExprKind, span: Option<Span>) -> Self {
        Expression { kind, span }
    }

    /// Convenience: create an `Rc<Expression>` without a span.
    pub fn rc(kind: ExprKind) -> Rc<Expression> {
        Rc::new(Expression::unspanned(kind))
    }

    /// Convenience: create an `Rc<Expression>` with a known span.
    pub fn rc_spanned(kind: ExprKind, span: Span) -> Rc<Expression> {
        Rc::new(Expression::new(kind, span))
    }

    /// Convenience: create an `Rc<Expression>` with an optional span.
    pub fn rc_with_span(kind: ExprKind, span: Option<Span>) -> Rc<Expression> {
        Rc::new(Expression::with_span(kind, span))
    }
}

/// Spans are intentionally ignored for equality — two expressions are equal
/// if they have the same structure, regardless of source location.
impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.kind.eq(&other.kind)
    }
}

impl core::hash::Hash for Expression {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl ASTDepth for Expression {
    fn ast_depth(&self) -> usize {
        self.kind.ast_depth()
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl From<ExprKind> for Expression {
    fn from(kind: ExprKind) -> Self {
        Expression::unspanned(kind)
    }
}

// ---------------------------------------------------------------------------
// Trait implementations for ExprKind
// ---------------------------------------------------------------------------

impl PartialEq for ExprKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExprKind::Commented(c1, e1), ExprKind::Commented(c2, e2)) => c1.eq(c2) && e1.eq(e2),
            (ExprKind::Lit(lit1), ExprKind::Lit(lit2)) => lit1.eq(lit2),
            (ExprKind::FnCall(fn_call1), ExprKind::FnCall(fn_call2)) => fn_call1.eq(fn_call2),
            (ExprKind::OpCall(id1, left1, right1), ExprKind::OpCall(id2, left2, right2)) => {
                id1.eq(id2) && left1.eq(left2) && right1.eq(right2)
            }
            (ExprKind::Var(id1), ExprKind::Var(id2)) => id1.eq(id2),
            (ExprKind::ConstOrTypeRef(id1), ExprKind::ConstOrTypeRef(id2)) => id1.eq(id2),
            (ExprKind::DBTypeRef(id1), ExprKind::DBTypeRef(id2)) => id1.eq(id2),
            (ExprKind::PropFnRef(id1), ExprKind::PropFnRef(id2)) => id1.eq(id2),
            (ExprKind::EdgeProp(expr1, edge1), ExprKind::EdgeProp(expr2, edge2)) => {
                expr1.eq(expr2) && edge1.eq(edge2)
            }
            (ExprKind::IfElse(if_else1), ExprKind::IfElse(if_else2)) => if_else1.eq(if_else2),
            (ExprKind::Let(l1), ExprKind::Let(l2)) => l1.eq(l2),
            (ExprKind::Lambda(l1), ExprKind::Lambda(l2)) => l1.eq(l2),
            (ExprKind::Query(q1), ExprKind::Query(q2)) => q1.eq(q2),
            (ExprKind::Symbol(id1), ExprKind::Symbol(id2)) => id1.eq(id2),
            (ExprKind::Quoted(e1), ExprKind::Quoted(e2)) => e1.eq(e2),
            (ExprKind::QuotedAST(e1), ExprKind::QuotedAST(e2)) => e1.eq(e2),
            (ExprKind::Unquoted(e1), ExprKind::Unquoted(e2)) => e1.eq(e2),
            (ExprKind::UnquotedAST(e1), ExprKind::UnquotedAST(e2)) => e1.eq(e2),
            (ExprKind::InlineFnDef(f1), ExprKind::InlineFnDef(f2)) => f1.eq(f2),
            (_, _) => false,
        }
    }
}

impl core::hash::Hash for ExprKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ExprKind::Commented(c, e) => {
                c.hash(state);
                e.hash(state)
            }
            ExprKind::Lit(lit_exp) => lit_exp.hash(state),
            ExprKind::FnCall(fn_call) => fn_call.hash(state),
            ExprKind::OpCall(id, left, right) => {
                id.hash(state);
                left.hash(state);
                right.hash(state)
            }
            ExprKind::Var(id) => id.hash(state),
            ExprKind::ConstOrTypeRef(id) => id.hash(state),
            ExprKind::DBTypeRef(id) => id.hash(state),
            ExprKind::PropFnRef(id) => id.hash(state),
            ExprKind::EdgeProp(expr, edge) => {
                expr.hash(state);
                edge.hash(state)
            }
            ExprKind::IfElse(if_else) => if_else.hash(state),
            ExprKind::Let(let_expr) => let_expr.hash(state),
            ExprKind::Lambda(lambda) => lambda.hash(state),
            ExprKind::Query(query) => query.hash(state),
            ExprKind::Symbol(id) => id.hash(state),
            ExprKind::Quoted(expr) => expr.hash(state),
            ExprKind::QuotedAST(expr) => expr.hash(state),
            ExprKind::Unquoted(expr) => expr.hash(state),
            ExprKind::UnquotedAST(expr) => expr.hash(state),
            ExprKind::InlineFnDef(fn_def) => fn_def.borrow().hash(state),
        }
    }
}

impl ASTDepth for ExprKind {
    fn ast_depth(&self) -> usize {
        match self {
            ExprKind::Commented(_, e) => 1 + e.ast_depth(),
            ExprKind::Lit(lit_exp) => lit_exp.ast_depth(),
            ExprKind::FnCall(fn_call) => fn_call.ast_depth(),
            ExprKind::OpCall(_id, left, right) => left.ast_depth() + right.ast_depth(),
            ExprKind::Var(_id) => 1,
            ExprKind::ConstOrTypeRef(_id) => 1,
            ExprKind::DBTypeRef(_id) => 1,
            ExprKind::PropFnRef(_id) => 1,
            ExprKind::EdgeProp(expr, _edge) => 1 + expr.ast_depth(),
            ExprKind::IfElse(if_else) => if_else.ast_depth(),
            ExprKind::Let(let_expr) => let_expr.ast_depth(),
            ExprKind::Lambda(lambda) => lambda.ast_depth(),
            ExprKind::Query(query) => query.ast_depth(),
            ExprKind::Symbol(_id) => 1,
            ExprKind::Quoted(expr) => 1 + expr.ast_depth(),
            ExprKind::QuotedAST(expr) => 1 + expr.ast_depth(),
            ExprKind::Unquoted(expr) => 1 + expr.ast_depth(),
            ExprKind::UnquotedAST(expr) => 1 + expr.ast_depth(),
            ExprKind::InlineFnDef(fn_def) => 1 + fn_def.borrow().ast_depth(),
        }
    }
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprKind::Commented(comment, exp) => {
                f.write_str("//")?;
                f.write_str(comment)?;
                f.write_str("\n")?;
                exp.fmt(f)
            }
            ExprKind::Lit(lit_exp) => lit_exp.fmt(f),
            ExprKind::FnCall(fn_call) => fn_call.fmt(f),
            ExprKind::OpCall(op_ident, left, right) => {
                f.write_str("(")?;
                left.fmt(f)?;
                f.write_str(" ")?;
                f.write_str(op_ident)?;
                f.write_str(" ")?;
                right.fmt(f)?;
                f.write_str(")")
            }
            ExprKind::Var(id) => id.fmt(f),
            ExprKind::ConstOrTypeRef(id) => f.write_str(id),
            ExprKind::DBTypeRef(id) => {
                f.write_str("@")?;
                f.write_str(id)
            }
            ExprKind::PropFnRef(id) => {
                f.write_str(".")?;
                f.write_str(id)
            }
            ExprKind::EdgeProp(expr, edge) => {
                expr.fmt(f)?;
                f.write_str("#")?;
                edge.fmt(f)
            }
            ExprKind::IfElse(if_else) => if_else.fmt(f),
            ExprKind::Let(let_expr) => let_expr.fmt(f),
            ExprKind::Lambda(lambda) => lambda.fmt(f),
            ExprKind::Query(query) => query.fmt(f),
            ExprKind::Symbol(id) => {
                f.write_str("^")?;
                f.write_str(id)
            }
            ExprKind::Quoted(expr) => display_quoted_expr(f, expr),
            ExprKind::QuotedAST(ast) => display_quoted_expr(f, ast),
            ExprKind::Unquoted(expr) => display_unquoted_expr(f, expr),
            ExprKind::UnquotedAST(ast) => display_unquoted_expr(f, ast),
            ExprKind::InlineFnDef(fn_def) => fn_def.borrow().fmt(f),
        }
    }
}

fn display_quoted_expr<Expr: Display>(
    f: &mut std::fmt::Formatter<'_>,
    expr: &Expr,
) -> std::fmt::Result {
    let expr_fmt = format!("{expr}");
    if expr_fmt.starts_with('(') && expr_fmt.ends_with(')') {
        f.write_str("^")?;
        expr_fmt.fmt(f)
    } else {
        f.write_str("^(")?;
        expr_fmt.fmt(f)?;
        f.write_str(")")
    }
}

fn display_unquoted_expr<Expr: Display>(
    f: &mut std::fmt::Formatter<'_>,
    expr: &Expr,
) -> std::fmt::Result {
    let expr_fmt = format!("{expr}");
    if expr_fmt.starts_with('(') && expr_fmt.ends_with(')') {
        f.write_str("~")?;
        expr_fmt.fmt(f)
    } else {
        f.write_str("~(")?;
        expr_fmt.fmt(f)?;
        f.write_str(")")
    }
}
