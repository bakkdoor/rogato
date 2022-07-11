use super::Identifier;
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModuleDef {
    id: Identifier,
    exports: ModuleExports,
}

impl ModuleDef {
    pub fn new(id: Identifier, exports: ModuleExports) -> ModuleDef {
        ModuleDef {
            id: id,
            exports: exports,
        }
    }
}

impl Display for ModuleDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("module {} {{ {} }}", self.id, self.exports))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModuleExports {
    exports: Box<Vec<Identifier>>,
}

impl ModuleExports {
    pub fn new(exports: Vec<Identifier>) -> Self {
        ModuleExports {
            exports: Box::new(exports),
        }
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
                .map(|e| format!("{}", e))
                .fold(String::from(""), |acc, fmt| {
                    if acc == "" {
                        fmt
                    } else {
                        format!("{}, {}", acc, fmt)
                    }
                });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}
