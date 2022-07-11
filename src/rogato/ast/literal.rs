use super::{expression::Expression, Identifier};
use crate::rogato::{db::val, interpreter::Evaluate, util::prepend_vec};
use serde_json::Value;
use std::fmt::Display;

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

impl<'a> Evaluate<'a, Value> for Literal {
    fn evaluate(&self, context: &mut crate::rogato::interpreter::EvalContext<'a>) -> Value {
        match self {
            Literal::Int64Lit(number) => val::number(*number),
            Literal::StringLit(string) => val::string(string),
            Literal::TupleLit(items) => val::array(prepend_vec(
                val::string(format!("rogato.Tuple.{}", items.len())),
                &mut items
                    .iter()
                    .map(|i| i.evaluate(context))
                    .collect::<Vec<Value>>(),
            )),
            Literal::ListLit(items) => {
                val::array(items.iter().map(|i| i.evaluate(context)).collect())
            }
            Literal::StructLit(_struct_id, props) => val::object(
                props
                    .iter()
                    .map(|(id, p)| (id.clone(), p.evaluate(context)))
                    .collect::<_>(),
            ),
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
