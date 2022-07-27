use super::{expression::Expression, ASTDepth, Identifier};
use crate::rogato::{
    db::{val, Value},
    interpreter::{EvalContext, Evaluate},
    util::indent,
};
use std::{fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Literal {
    Int64(i64),
    String(String),
    Tuple(TupleItems<Expression>),
    List(TupleItems<Expression>),
    Struct(Identifier, Rc<StructProps>),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int64(num) => f.write_fmt(format_args!("{}", num)),
            Literal::String(string) => f.write_fmt(format_args!("\"{}\"", string)),
            Literal::Tuple(items) => {
                if items.ast_depth() > 6 {
                    let items_str = format!("{}", items);
                    if items_str.split('\n').count() == 1 {
                        f.write_fmt(format_args!("{{ {} }}", items))
                    } else {
                        f.write_fmt(format_args!("{{\n{}\n}}", indent(items)))
                    }
                } else {
                    f.write_fmt(format_args!("{{ {} }}", items))
                }
            }
            Literal::List(items) => {
                if items.ast_depth() > 6 {
                    let items_str = format!("{}", items);
                    if items_str.split('\n').count() == 1 {
                        f.write_fmt(format_args!("[ {} ]", items))
                    } else {
                        f.write_fmt(format_args!("[\n{}\n]", indent(items)))
                    }
                } else {
                    f.write_fmt(format_args!("[ {} ]", items))
                }
            }
            Literal::Struct(id, props) => f.write_fmt(format_args!("{}{{ {} }}", id, props)),
        }
    }
}

impl ASTDepth for Literal {
    fn ast_depth(&self) -> usize {
        match self {
            Literal::Int64(_) => 1,
            Literal::String(_) => 1,
            Literal::Tuple(items) => 1 + items.iter().map(|i| i.ast_depth()).sum::<usize>(),
            Literal::List(items) => 1 + items.iter().map(|i| i.ast_depth()).sum::<usize>(),
            Literal::Struct(_id, props) => {
                1 + props
                    .iter()
                    .map(|(_name, val)| val.ast_depth())
                    .sum::<usize>()
            }
        }
    }
}

impl Evaluate<Value> for Literal {
    fn evaluate(&self, context: &mut EvalContext) -> Value {
        match self {
            Literal::Int64(number) => val::number(*number),
            Literal::String(string) => val::string(string),
            Literal::Tuple(items) => {
                let mut values = items
                    .iter()
                    .map(|i| i.evaluate(context))
                    .collect::<Vec<Value>>();
                values.insert(0, val::string(format!("rogato.Tuple.{}", items.len())));
                val::array(values)
            }
            Literal::List(items) => val::array(items.iter().map(|i| i.evaluate(context)).collect()),
            Literal::Struct(_struct_id, props) => val::object(
                props
                    .iter()
                    .map(|(id, p)| (id.clone(), p.evaluate(context)))
                    .collect(),
            ),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TupleItems<I> {
    items: Vec<Rc<I>>,
}

impl<I: Display> TupleItems<I> {
    pub fn new(first: I, rest: Vec<I>) -> Self {
        let mut items = vec![Rc::new(first)];
        for item in rest {
            items.push(Rc::new(item))
        }
        Self::from(items)
    }

    pub fn from(items: Vec<Rc<I>>) -> Self {
        TupleItems { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<I>> {
        self.items.iter()
    }
}

impl<I: Display + ASTDepth> Display for TupleItems<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent_items = self.items.iter().any(|i| i.ast_depth() > 6);
        let fmt_str = self.items.iter().fold(String::from(""), |acc, item| {
            if acc.is_empty() {
                format!("{}", item)
            } else if indent_items {
                format!("{},\n{}", acc, item)
            } else {
                format!("{}, {}", acc, item)
            }
        });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

impl<I: ASTDepth> ASTDepth for TupleItems<I> {
    fn ast_depth(&self) -> usize {
        1 + self.items.iter().map(|i| i.ast_depth()).sum::<usize>()
    }
}

impl<T: Evaluate<Value>> Evaluate<Value> for TupleItems<T> {
    fn evaluate(&self, context: &mut EvalContext) -> Value {
        val::array(self.items.iter().map(|i| i.evaluate(context)).collect())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructProps {
    props: Vec<(Identifier, Rc<Expression>)>,
}

impl StructProps {
    pub fn new(
        first: (Identifier, Rc<Expression>),
        rest: Vec<(Identifier, Rc<Expression>)>,
    ) -> Self {
        let mut boxed_props = Vec::new();
        let (f_id, f_expr) = first;
        boxed_props.push((f_id, f_expr));
        for (id, expr) in rest {
            boxed_props.push((id, expr))
        }
        Self::from(boxed_props)
    }

    pub fn from(props: Vec<(Identifier, Rc<Expression>)>) -> Self {
        StructProps { props }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.props.len()
    }

    pub fn iter(&self) -> std::slice::Iter<(String, Rc<Expression>)> {
        self.props.iter()
    }
}

impl Display for StructProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self.iter().fold(String::from(""), |acc, (id, expr)| {
            if acc.is_empty() {
                format!("{}: {}", id, expr)
            } else {
                format!("{}, {}: {}", acc, id, expr)
            }
        });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
