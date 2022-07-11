pub use super::fn_call::FnCallArgs;
pub use super::fn_def::FnDefArgs;
pub use super::lambda::{Lambda, LambdaArgs};
pub use super::let_expression::{LetBindings, LetExpression};
pub use super::query::{Query, QueryBinding, QueryBindings, QueryGuards};
use super::Identifier;
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Commented(String, Box<Expression>),
    Lit(Literal),
    Sum(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    FnCall(Identifier, Box<FnCallArgs>),
    OpCall(Identifier, Box<Expression>, Box<Expression>),
    Var(Identifier),
    ConstOrTypeRef(Identifier),
    PropFnRef(Identifier),
    EdgeProp(Box<Expression>, Identifier),
    Let(Box<LetExpression>),
    Lambda(Box<Lambda>),
    Query(Box<Query>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Commented(comment, exp) => {
                f.write_fmt(format_args!("//{}\n{}", comment, exp))
            }
            Expression::Lit(lit_exp) => lit_exp.fmt(f),
            Expression::Sum(a, b) => f.write_fmt(format_args!("({} + {})", a, b)),
            Expression::Product(a, b) => f.write_fmt(format_args!("({} * {})", a, b)),
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
            Expression::Let(let_expr) => f.write_fmt(format_args!("{}", let_expr)),
            Expression::Lambda(lambda) => f.write_fmt(format_args!("{}", lambda)),
            Expression::Query(query) => f.write_fmt(format_args!("{}", query)),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Literal {
    Int64Lit(i64),
    StringLit(Box<String>),
    TupleLit(TupleItems<Expression>),
    ListLit(TupleItems<Expression>),
    StructLit(Identifier, Box<StructProps>),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int64Lit(num) => f.write_fmt(format_args!("{}", num)),
            Literal::StringLit(string) => f.write_fmt(format_args!("{}", string)),
            Literal::TupleLit(items) => f.write_fmt(format_args!("{{ {} }}", items)),
            Literal::ListLit(items) => f.write_fmt(format_args!("[ {} ]", items)),
            Literal::StructLit(id, props) => f.write_fmt(format_args!("{}{{ {} }}", id, props)),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TupleItems<I> {
    items: Vec<Box<I>>,
}

impl<I: Display> TupleItems<I> {
    pub fn new(first: I, rest: Vec<I>) -> Self {
        let mut items = Vec::new();
        items.push(Box::new(first));
        for item in rest {
            items.push(Box::new(item))
        }
        Self::from(items)
    }

    pub fn from(items: Vec<Box<I>>) -> Self {
        TupleItems { items: items }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::slice::Iter<Box<I>> {
        self.items.iter()
    }
}

impl<I: Display> Display for TupleItems<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self.items.iter().fold(String::from(""), |acc, item| {
            if acc == "" {
                format!("{}", item)
            } else {
                format!("{}, {}", acc, item)
            }
        });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructProps {
    props: Vec<(Identifier, Box<Expression>)>,
}

impl StructProps {
    pub fn new(first: (Identifier, Expression), rest: Vec<(Identifier, Expression)>) -> Self {
        let mut boxed_props = Vec::new();
        let (f_id, f_expr) = first;
        boxed_props.push((f_id, Box::new(f_expr)));
        for (id, expr) in rest {
            boxed_props.push((id, Box::new(expr)))
        }
        Self::from(boxed_props)
    }

    pub fn from(props: Vec<(Identifier, Box<Expression>)>) -> Self {
        StructProps { props: props }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.props.len()
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::slice::Iter<(String, Box<Expression>)> {
        self.props.iter()
    }
}

impl Display for StructProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self.props.iter().fold(String::from(""), |acc, (id, expr)| {
            if acc == "" {
                format!("{}: {}", id, expr)
            } else {
                format!("{}, {}: {}", acc, id, expr)
            }
        });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
