use indent_write::indentable::Indentable;

pub use super::fn_call::FnCallArgs;
pub use super::fn_def::FnDefArgs;
pub use super::query::{QueryBinding, QueryBindings, QueryGuards};
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
    Let(Box<LetBindings>, Box<Expression>),
    Lambda(Box<LambdaArgs<Identifier>>, Box<Expression>),
    Query(Box<QueryBindings>, Box<QueryGuards>, Box<Expression>),
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
            Expression::Let(bindings, body) => f.write_fmt(format_args!(
                "let\n{}\nin\n{}",
                bindings.indented("    "),
                body.indented("    ")
            )),
            Expression::Lambda(args, body) => f.write_fmt(format_args!("({} -> {})", args, body)),
            Expression::Query(query, guards, production) => {
                if guards.is_empty() {
                    f.write_fmt(format_args!("{}\n!> {}", query, production))
                } else {
                    f.write_fmt(format_args!("{}\n{}\n!> {}", query, guards, production))
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetBindings {
    bindings: Vec<(Identifier, Expression)>,
}

impl LetBindings {
    pub fn new(bindings: Vec<(Identifier, Expression)>) -> Box<LetBindings> {
        Box::new(LetBindings { bindings: bindings })
    }

    pub fn iter(&self) -> std::slice::Iter<(String, Expression)> {
        self.bindings.iter()
    }
}

impl Display for LetBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self
            .bindings
            .iter()
            .map(|(ident, expr)| format!("{} = {}", ident, expr))
            .fold(String::from(""), |acc, fmt| {
                if acc == "" {
                    fmt
                } else {
                    format!("{},\n{}", acc, fmt)
                }
            });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LambdaArgs<A: Display> {
    args: Vec<A>,
}

impl<A: Display> LambdaArgs<A> {
    pub fn new(args: Vec<A>) -> Box<LambdaArgs<A>> {
        Box::new(LambdaArgs { args: args })
    }
}

impl<A: Display> Display for LambdaArgs<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self.args.iter().fold(String::from(""), |acc, arg| {
            if acc == "" {
                arg.to_string()
            } else {
                format!("{} {}", acc, arg)
            }
        });

        f.write_fmt(format_args!("{}", fmt_str))
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
