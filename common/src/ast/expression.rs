pub use super::fn_call::FnCallArgs;
use super::fn_def::FnDef;
pub use super::fn_def::FnDefArgs;
pub use super::lambda::{Lambda, LambdaArgs};
pub use super::let_expression::{LetBindings, LetExpression};
pub use super::literal::*;
pub use super::query::{Query, QueryBinding, QueryBindings, QueryGuards};
use super::{ASTDepth, Identifier, AST};
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Commented(String, Rc<Expression>),
    Lit(Literal),
    FnCall(Identifier, FnCallArgs),
    OpCall(Identifier, Rc<Expression>, Rc<Expression>),
    Var(Identifier),
    ConstOrTypeRef(Identifier),
    PropFnRef(Identifier),
    EdgeProp(Rc<Expression>, Identifier),
    Let(LetExpression),
    Lambda(Lambda),
    Query(Query),
    Symbol(Identifier),
    Quoted(Rc<Expression>),
    QuotedAST(Rc<AST>),
    Unquoted(Rc<Expression>),
    UnquotedAST(Rc<AST>),
    InlineFnDef(Rc<FnDef>),
}

impl ASTDepth for Expression {
    fn ast_depth(&self) -> usize {
        match self {
            Expression::Commented(_, e) => 1 + e.ast_depth(),
            Expression::Lit(lit_exp) => lit_exp.ast_depth(),
            Expression::FnCall(_id, args) => args.iter().map(|a| a.ast_depth()).sum(),
            Expression::OpCall(_id, left, right) => left.ast_depth() + right.ast_depth(),
            Expression::Var(_id) => 1,
            Expression::ConstOrTypeRef(_id) => 1,
            Expression::PropFnRef(_id) => 1,
            Expression::EdgeProp(expr, _edge) => 1 + expr.ast_depth(),
            Expression::Let(let_expr) => let_expr.ast_depth(),
            Expression::Lambda(lambda) => lambda.ast_depth(),
            Expression::Query(query) => query.ast_depth(),
            Expression::Symbol(_id) => 1,
            Expression::Quoted(expr) => 1 + expr.ast_depth(),
            Expression::QuotedAST(expr) => 1 + expr.ast_depth(),
            Expression::Unquoted(expr) => 1 + expr.ast_depth(),
            Expression::UnquotedAST(expr) => 1 + expr.ast_depth(),
            Expression::InlineFnDef(fn_def) => 1 + fn_def.ast_depth(),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Commented(comment, exp) => {
                f.write_fmt(format_args!("//{}\n{}", comment, exp))
            }
            Expression::Lit(lit_exp) => lit_exp.fmt(f),
            Expression::FnCall(fn_ident, args) => {
                f.write_fmt(format_args!("({}{})", fn_ident, args))
            }
            Expression::OpCall(op_ident, left, right) => {
                f.write_fmt(format_args!("({} {} {})", left, op_ident, right))
            }
            Expression::Var(id) => f.write_str(id),
            Expression::ConstOrTypeRef(id) => f.write_str(id),
            Expression::PropFnRef(id) => f.write_fmt(format_args!(".{}", id)),
            Expression::EdgeProp(id, edge) => f.write_fmt(format_args!("{}#{}", id, edge)),
            Expression::Let(let_expr) => let_expr.fmt(f),
            Expression::Lambda(lambda) => lambda.fmt(f),
            Expression::Query(query) => query.fmt(f),
            Expression::Symbol(id) => f.write_fmt(format_args!("^{}", id)),
            Expression::Quoted(expr) => display_quoted(f, expr),
            Expression::QuotedAST(ast) => display_quoted(f, ast),
            Expression::Unquoted(expr) => display_unquoted(f, expr),
            Expression::UnquotedAST(ast) => display_unquoted(f, ast),
            Expression::InlineFnDef(fn_def) => f.write_fmt(format_args!("{}", fn_def)),
        }
    }
}

fn display_quoted<Expr: Display>(f: &mut std::fmt::Formatter<'_>, expr: &Expr) -> std::fmt::Result {
    let expr_fmt = format!("{}", expr);
    if expr_fmt.starts_with('(') && expr_fmt.ends_with(')') {
        f.write_fmt(format_args!("^{}", expr_fmt))
    } else {
        f.write_fmt(format_args!("^({})", expr_fmt))
    }
}

fn display_unquoted<Expr: Display>(
    f: &mut std::fmt::Formatter<'_>,
    expr: &Expr,
) -> std::fmt::Result {
    let expr_fmt = format!("{}", expr);
    if expr_fmt.starts_with('(') && expr_fmt.ends_with(')') {
        f.write_fmt(format_args!("~{}", expr_fmt))
    } else {
        f.write_fmt(format_args!("~({})", expr_fmt))
    }
}