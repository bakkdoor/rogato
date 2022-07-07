use super::Identifier;
use std::fmt::Display;

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
