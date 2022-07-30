use crate::rogato::{
    db::val,
    db::val::Value,
    interpreter::{EvalContext, Evaluate},
    util::indent,
};

use super::{expression::Expression, ASTDepth, Identifier};
use std::{fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnDef {
    id: Identifier,
    args: FnDefArgs,
    body: Rc<Expression>,
}

impl FnDef {
    pub fn new(id: Identifier, args: FnDefArgs, body: Rc<Expression>) -> FnDef {
        FnDef { id, args, body }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn args(&self) -> &FnDefArgs {
        &self.args
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }
}

impl Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "let {}{} =\n{}",
            self.id,
            self.args,
            indent(self.body.to_owned())
        ))
    }
}

impl ASTDepth for FnDef {
    fn ast_depth(&self) -> usize {
        1 + self.args.len() + self.body.ast_depth()
    }
}

impl Evaluate<Value> for FnDef {
    fn evaluate(&self, context: &mut EvalContext) -> Value {
        context.define_fn(FnDef::new(
            self.id.clone(),
            self.args.clone(),
            self.body.clone(),
        ));
        val::object(vec![
            ("type", val::string("Fn")),
            ("name", val::string(self.id.to_string())),
            (
                "args",
                Value::Array(self.args.iter().map(val::string).collect::<Vec<Value>>()),
            ),
            ("body", self.body.evaluate(context)),
        ])
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

    pub fn iter(&self) -> std::slice::Iter<String> {
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
