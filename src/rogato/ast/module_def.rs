use crate::rogato::{
    db::{val, Value},
    interpreter::{EvalContext, Evaluate},
};

use super::{ASTDepth, Identifier};
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModuleDef {
    id: Identifier,
    exports: ModuleExports,
}

impl ModuleDef {
    pub fn new(id: Identifier, exports: ModuleExports) -> ModuleDef {
        ModuleDef { id, exports }
    }
}

impl Display for ModuleDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("module {} {{ {} }}", self.id, self.exports))
    }
}

impl ASTDepth for ModuleDef {
    fn ast_depth(&self) -> usize {
        1 + self.exports.ast_depth()
    }
}

impl Evaluate<Value> for ModuleDef {
    fn evaluate(&self, _context: &mut EvalContext) -> Value {
        val::object(vec![
            ("type", Value::String("Module".to_string())),
            ("name", Value::String(self.id.to_string())),
            (
                "exports",
                Value::Array(
                    self.exports
                        .iter()
                        .map(|e| Value::String(e.to_string()))
                        .collect::<Vec<Value>>(),
                ),
            ),
        ])
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModuleExports {
    exports: Vec<Identifier>,
}

impl ModuleExports {
    pub fn new(exports: Vec<Identifier>) -> Self {
        ModuleExports { exports }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.exports.len()
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::slice::Iter<String> {
        self.exports.iter()
    }
}

impl Display for ModuleExports {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str =
            self.exports
                .iter()
                .map(|e| e.to_string())
                .fold(String::from(""), |acc, fmt| {
                    if acc.is_empty() {
                        fmt
                    } else {
                        format!("{}, {}", acc, fmt)
                    }
                });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

impl ASTDepth for ModuleExports {
    fn ast_depth(&self) -> usize {
        self.exports.len()
    }
}
