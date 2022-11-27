pub use super::fn_call::{FnCall, FnCallArgs};
use super::fn_def::FnDef;
pub use super::fn_def::FnDefArgs;
pub use super::if_else::IfElse;
pub use super::lambda::{Lambda, LambdaArgs};
pub use super::let_expression::{LetBindings, LetExpression};
pub use super::literal::*;
pub use super::query::{Query, QueryBinding, QueryBindings, QueryGuards};
use super::{ASTDepth, Identifier, AST};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, Eq, Debug)]
pub enum Expression {
    Commented(String, Rc<Expression>),
    Lit(Literal),
    FnCall(FnCall),
    OpCall(Identifier, Rc<Expression>, Rc<Expression>),
    Var(Identifier),
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

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expression::Commented(c1, e1), Expression::Commented(c2, e2)) => {
                c1.eq(c2) && e1.eq(e2)
            }
            (Expression::Lit(lit1), Expression::Lit(lit2)) => lit1.eq(lit2),
            (Expression::FnCall(fn_call1), Expression::FnCall(fn_call2)) => fn_call1.eq(fn_call2),
            (Expression::OpCall(id1, left1, right1), Expression::OpCall(id2, left2, right2)) => {
                id1.eq(id2) && left1.eq(left2) && right1.eq(right2)
            }
            (Expression::Var(id1), Expression::Var(id2)) => id1.eq(id2),
            (Expression::ConstOrTypeRef(id1), Expression::ConstOrTypeRef(id2)) => id1.eq(id2),
            (Expression::DBTypeRef(id1), Expression::DBTypeRef(id2)) => id1.eq(id2),
            (Expression::PropFnRef(id1), Expression::PropFnRef(id2)) => id1.eq(id2),
            (Expression::EdgeProp(expr1, edge1), Expression::EdgeProp(expr2, edge2)) => {
                expr1.eq(expr2) && edge1.eq(edge2)
            }
            (Expression::IfElse(if_else1), Expression::IfElse(if_else2)) => if_else1.eq(if_else2),
            (Expression::Let(l1), Expression::Let(l2)) => l1.eq(l2),
            (Expression::Lambda(l1), Expression::Lambda(l2)) => l1.eq(l2),
            (Expression::Query(q1), Expression::Query(q2)) => q1.eq(q2),
            (Expression::Symbol(id1), Expression::Symbol(id2)) => id1.eq(id2),
            (Expression::Quoted(e1), Expression::Quoted(e2)) => e1.eq(e2),
            (Expression::QuotedAST(e1), Expression::QuotedAST(e2)) => e1.eq(e2),
            (Expression::Unquoted(e1), Expression::Unquoted(e2)) => e1.eq(e2),
            (Expression::UnquotedAST(e1), Expression::UnquotedAST(e2)) => e1.eq(e2),
            (Expression::InlineFnDef(f1), Expression::InlineFnDef(f2)) => f1.eq(f2),
            (_, _) => false,
        }
    }
}

impl core::hash::Hash for Expression {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Expression::Commented(c, e) => {
                c.hash(state);
                e.hash(state)
            }
            Expression::Lit(lit_exp) => lit_exp.hash(state),
            Expression::FnCall(fn_call) => fn_call.hash(state),
            Expression::OpCall(id, left, right) => {
                id.hash(state);
                left.hash(state);
                right.hash(state)
            }
            Expression::Var(id) => id.hash(state),
            Expression::ConstOrTypeRef(id) => id.hash(state),
            Expression::DBTypeRef(id) => id.hash(state),
            Expression::PropFnRef(id) => id.hash(state),
            Expression::EdgeProp(expr, edge) => {
                expr.hash(state);
                edge.hash(state)
            }
            Expression::IfElse(if_else) => if_else.hash(state),
            Expression::Let(let_expr) => let_expr.hash(state),
            Expression::Lambda(lambda) => lambda.hash(state),
            Expression::Query(query) => query.hash(state),
            Expression::Symbol(id) => id.hash(state),
            Expression::Quoted(expr) => expr.hash(state),
            Expression::QuotedAST(expr) => expr.hash(state),
            Expression::Unquoted(expr) => expr.hash(state),
            Expression::UnquotedAST(expr) => expr.hash(state),
            Expression::InlineFnDef(fn_def) => fn_def.borrow().hash(state),
        }
    }
}

impl ASTDepth for Expression {
    fn ast_depth(&self) -> usize {
        match self {
            Expression::Commented(_, e) => 1 + e.ast_depth(),
            Expression::Lit(lit_exp) => lit_exp.ast_depth(),
            Expression::FnCall(fn_call) => fn_call.ast_depth(),
            Expression::OpCall(_id, left, right) => left.ast_depth() + right.ast_depth(),
            Expression::Var(_id) => 1,
            Expression::ConstOrTypeRef(_id) => 1,
            Expression::DBTypeRef(_id) => 1,
            Expression::PropFnRef(_id) => 1,
            Expression::EdgeProp(expr, _edge) => 1 + expr.ast_depth(),
            Expression::IfElse(if_else) => if_else.ast_depth(),
            Expression::Let(let_expr) => let_expr.ast_depth(),
            Expression::Lambda(lambda) => lambda.ast_depth(),
            Expression::Query(query) => query.ast_depth(),
            Expression::Symbol(_id) => 1,
            Expression::Quoted(expr) => 1 + expr.ast_depth(),
            Expression::QuotedAST(expr) => 1 + expr.ast_depth(),
            Expression::Unquoted(expr) => 1 + expr.ast_depth(),
            Expression::UnquotedAST(expr) => 1 + expr.ast_depth(),
            Expression::InlineFnDef(fn_def) => 1 + fn_def.borrow().ast_depth(),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Commented(comment, exp) => {
                f.write_str("//")?;
                f.write_str(comment)?;
                f.write_str("\n")?;
                exp.fmt(f)
            }
            Expression::Lit(lit_exp) => lit_exp.fmt(f),
            Expression::FnCall(fn_call) => fn_call.fmt(f),
            Expression::OpCall(op_ident, left, right) => {
                f.write_str("(")?;
                left.fmt(f)?;
                f.write_str(" ")?;
                f.write_str(op_ident)?;
                f.write_str(" ")?;
                right.fmt(f)?;
                f.write_str(")")
            }
            Expression::Var(id) => f.write_str(id),
            Expression::ConstOrTypeRef(id) => f.write_str(id),
            Expression::DBTypeRef(id) => {
                f.write_str("@")?;
                f.write_str(id)
            }
            Expression::PropFnRef(id) => {
                f.write_str(".")?;
                f.write_str(id)
            }
            Expression::EdgeProp(expr, edge) => {
                expr.fmt(f)?;
                f.write_str("#")?;
                edge.fmt(f)
            }
            Expression::IfElse(if_else) => if_else.fmt(f),
            Expression::Let(let_expr) => let_expr.fmt(f),
            Expression::Lambda(lambda) => lambda.fmt(f),
            Expression::Query(query) => query.fmt(f),
            Expression::Symbol(id) => {
                f.write_str("^")?;
                f.write_str(id)
            }
            Expression::Quoted(expr) => display_quoted_expr(f, expr),
            Expression::QuotedAST(ast) => display_quoted_expr(f, ast),
            Expression::Unquoted(expr) => display_unquoted_expr(f, expr),
            Expression::UnquotedAST(ast) => display_unquoted_expr(f, ast),
            Expression::InlineFnDef(fn_def) => fn_def.borrow().fmt(f),
        }
    }
}

fn display_quoted_expr<Expr: Display>(
    f: &mut std::fmt::Formatter<'_>,
    expr: &Expr,
) -> std::fmt::Result {
    let expr_fmt = format!("{}", expr);
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
    let expr_fmt = format!("{}", expr);
    if expr_fmt.starts_with('(') && expr_fmt.ends_with(')') {
        f.write_str("~")?;
        expr_fmt.fmt(f)
    } else {
        f.write_str("~(")?;
        expr_fmt.fmt(f)?;
        f.write_str(")")
    }
}
