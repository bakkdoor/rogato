pub use super::fn_call::FnCallArgs;
use super::fn_def::FnDef;
pub use super::fn_def::FnDefArgs;
pub use super::if_else::IfElse;
pub use super::lambda::{Lambda, LambdaArgs};
pub use super::let_expression::{LetBindings, LetExpression};
pub use super::literal::*;
pub use super::query::{Query, QueryBinding, QueryBindings, QueryGuards};
use super::{ASTDepth, Identifier, AST};
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Expression {
    Commented(String, Rc<Expression>),
    Lit(Literal),
    FnCall(Identifier, FnCallArgs),
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
            Expression::InlineFnDef(fn_def) => 1 + fn_def.ast_depth(),
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
            Expression::FnCall(fn_ident, args) => {
                f.write_str("(")?;
                fn_ident.fmt(f)?;
                f.write_str(" ")?;
                args.fmt(f)?;
                f.write_str(")")
            }
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
            Expression::InlineFnDef(fn_def) => fn_def.fmt(f),
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
