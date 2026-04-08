use std::{collections::HashMap, fmt, rc::Rc};

use rogato_common::ast::{type_expression::TypeExpression, Identifier, VarIdentifier};

#[derive(Clone)]
pub struct TypeEnvironment {
    variables: HashMap<VarIdentifier, Rc<TypeExpression>>,
    functions: HashMap<Identifier, FnSignature>,
    type_defs: HashMap<Identifier, Rc<TypeExpression>>,
}

#[derive(Clone, Debug)]
pub struct FnSignature {
    pub name: Identifier,
    pub arg_types: Vec<Rc<TypeExpression>>,
    pub return_type: Rc<TypeExpression>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        TypeEnvironment {
            variables: HashMap::new(),
            functions: HashMap::new(),
            type_defs: HashMap::new(),
        }
    }

    pub fn insert_variable(&mut self, id: VarIdentifier, type_expr: Rc<TypeExpression>) {
        self.variables.insert(id, type_expr);
    }

    pub fn lookup_variable(&self, id: &VarIdentifier) -> Option<&Rc<TypeExpression>> {
        self.variables.get(id)
    }

    pub fn insert_function(&mut self, sig: FnSignature) {
        self.functions.insert(sig.name.clone(), sig);
    }

    pub fn lookup_function(&self, name: &Identifier) -> Option<&FnSignature> {
        self.functions.get(name)
    }

    pub fn insert_type_def(&mut self, id: Identifier, type_expr: Rc<TypeExpression>) {
        self.type_defs.insert(id, type_expr);
    }

    pub fn lookup_type_def(&self, id: &Identifier) -> Option<&Rc<TypeExpression>> {
        self.type_defs.get(id)
    }

    pub fn new_scope(&self) -> TypeEnvironment {
        self.clone()
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        TypeEnvironment::new()
    }
}

impl fmt::Debug for TypeEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeEnvironment")
            .field("variables", &self.variables.keys().collect::<Vec<_>>())
            .field("functions", &self.functions.keys().collect::<Vec<_>>())
            .finish()
    }
}
