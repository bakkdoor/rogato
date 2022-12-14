use super::{module::Module, ValueRef};
use rogato_common::{
    ast::{fn_def::FnDef, type_expression::TypeDef, Identifier, VarIdentifier},
    flame_guard,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[cfg(feature = "flame_it")]
use flamer::flame;

type FuncId = Identifier;
type TypeId = Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ImportedIdentifier {
    Func(FuncId),
    Type(TypeId),
    AliasedFunc(FuncId, FuncId), // imported_id, local_alias
    AliasedType(TypeId, TypeId), // imported_id, local_alias
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Imports {
    All,
    Specific(Vec<ImportedIdentifier>),
}

impl Imports {
    pub fn import_ids(&self) -> Vec<Identifier> {
        match self {
            Imports::All => vec!["*".into()],
            Imports::Specific(ids) => ids
                .iter()
                .map(|id| match id {
                    ImportedIdentifier::Func(id) => id.clone(),
                    ImportedIdentifier::Type(id) => id.clone(),
                    ImportedIdentifier::AliasedFunc(id, alias) => {
                        format!("{} as {}", id, alias).into()
                    }
                    ImportedIdentifier::AliasedType(id, alias) => {
                        format!("{} as {}", id, alias).into()
                    }
                })
                .collect(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ImportedModules {
    imports: HashMap<Identifier, Imports>,
}

impl ImportedModules {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, Identifier, Imports> {
        self.imports.iter()
    }

    pub fn import(&mut self, module: &Module, imports: Imports) -> &mut Self {
        self.imports.insert(module.id(), imports);
        self
    }
}

impl FromIterator<(Identifier, Imports)> for ImportedModules {
    fn from_iter<T: IntoIterator<Item = (Identifier, Imports)>>(iter: T) -> Self {
        ImportedModules {
            imports: HashMap::from_iter(iter.into_iter()),
        }
    }
}

impl Default for ImportedModules {
    fn default() -> Self {
        ImportedModules::from_iter([(
            "Std.List".into(),
            Imports::Specific(vec![ImportedIdentifier::Func("map".into())]),
        )])
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct State {
    parent: Option<Environment>,
    variables: HashMap<VarIdentifier, ValueRef>,
    modules: Rc<RefCell<HashMap<Identifier, Module>>>,
    imported_modules: ImportedModules,
    aliased_modules: HashMap<Identifier, Identifier>,
    current_module_name: Identifier,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Environment {
    state: Rc<RefCell<State>>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::new()
    }
}

impl Environment {
    pub fn new() -> Environment {
        let modules = Rc::new(RefCell::new(HashMap::new()));
        let mod_name: Identifier = "Std".into();
        modules
            .borrow_mut()
            .insert(mod_name.clone(), Module::new(&mod_name));

        let state = State {
            parent: None,
            variables: HashMap::new(),
            modules,
            imported_modules: ImportedModules::new(),
            aliased_modules: HashMap::new(),
            current_module_name: mod_name,
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn new_with_imported_modules(imported_modules: ImportedModules) -> Environment {
        let modules = Rc::new(RefCell::new(HashMap::new()));
        let mod_name: Identifier = "Std".into();
        modules
            .borrow_mut()
            .insert(mod_name.clone(), Module::new(&mod_name));

        let state = State {
            parent: None,
            variables: HashMap::new(),
            modules,
            imported_modules,
            aliased_modules: HashMap::new(),
            current_module_name: mod_name,
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    #[inline]
    pub fn clear_variables(&self) {
        let mut state = self.state.borrow_mut();
        state.variables.clear();
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn child(&self) -> Environment {
        let curr_state = self.state.borrow();
        let state = State {
            parent: Some(self.clone()),
            variables: HashMap::new(),
            modules: Rc::clone(&curr_state.modules),
            imported_modules: self.imported_modules(),
            aliased_modules: HashMap::new(),
            current_module_name: curr_state.current_module_name.clone(),
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn child_with_imported_modules(&self, imported_modules: ImportedModules) -> Environment {
        let curr_state = self.state.borrow();
        let state = State {
            parent: Some(self.clone()),
            variables: HashMap::new(),
            modules: Rc::clone(&curr_state.modules),
            imported_modules,
            aliased_modules: HashMap::new(),
            current_module_name: curr_state.current_module_name.clone(),
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn imported_modules(&self) -> ImportedModules {
        self.state.borrow().imported_modules.clone()
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn import(&mut self, module: &Module, imports: Imports) {
        self.state
            .borrow_mut()
            .imported_modules
            .import(module, imports);
    }

    pub fn alias_module(&mut self, module: &Module, as_str: &str) {
        self.state
            .borrow_mut()
            .aliased_modules
            .insert(Identifier::from(as_str), module.id());
    }

    #[cfg_attr(feature = "flame_it", flame)]
    #[inline]
    pub fn lookup_module_alias(&self, id: &Identifier) -> Option<Identifier> {
        let state = self.state.borrow();
        let opt_mod_name = state.aliased_modules.get(id).map(Identifier::clone);

        match (&opt_mod_name, &state.parent) {
            (Some(_), _) => opt_mod_name,
            (None, Some(parent)) => parent.lookup_module_alias(id),
            (None, None) => None,
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    #[inline]
    pub fn define_var(&mut self, id: &VarIdentifier, val: ValueRef) {
        flame_guard!("= {} {}", id, &val);

        self.state.borrow_mut().variables.insert(id.clone(), val);
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn lookup_var(&self, id: &VarIdentifier) -> Option<ValueRef> {
        flame_guard!("🔎 {}", &id);

        let state = self.state.borrow();
        let res = match state.variables.get(id) {
            Some(expr) => Some(expr.clone()),
            None => match &state.parent {
                Some(parent_env) => parent_env.lookup_var(id),
                None => None,
            },
        };
        res
    }

    pub fn define_module(&mut self, module: Module) {
        let id = module.id();
        let state = self.state.borrow_mut();
        state.modules.borrow_mut().insert(id, module);
    }

    pub fn current_module(&self) -> Module {
        let state = self.state.borrow();
        self.lookup_module(&state.current_module_name)
            .expect("current_module should be set")
    }

    pub fn set_current_module(&mut self, module_id: Identifier) {
        let mut state = self.state.borrow_mut();
        state.current_module_name = module_id;
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn lookup_module(&self, id: &Identifier) -> Option<Module> {
        match self.state.borrow().modules.borrow().get(id) {
            Some(module) => Some(module.clone()),
            None => match &self.state.borrow().parent {
                Some(parent_env) => parent_env.lookup_module(id),
                None => None,
            },
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn lookup_const(&self, id: &Identifier) -> Option<ValueRef> {
        if let Some((module_id, fn_id)) = self.qualified_lookup(id) {
            return self
                .lookup_module(&module_id)
                .and_then(|m| m.lookup_const(&fn_id));
        }

        match self
            .lookup_module_for_const(id)
            .and_then(|m| m.lookup_const(id))
        {
            Some(val) => Some(val),
            None => {
                let state = self.state.borrow();
                match &state.parent {
                    Some(parent_env) => parent_env.lookup_const(id),
                    None => None,
                }
            }
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        if let Some((module_id, fn_id)) = self.qualified_lookup(id) {
            return self
                .lookup_module(&module_id)
                .and_then(|m| m.lookup_type(&fn_id));
        }

        match self
            .lookup_module_for_type(id)
            .and_then(|m| m.lookup_type(id))
        {
            Some(type_) => Some(type_),
            None => {
                let state = self.state.borrow();
                match &state.parent {
                    Some(parent_env) => parent_env.lookup_type(id),
                    None => None,
                }
            }
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn lookup_fn(&self, id: &Identifier) -> Option<Rc<RefCell<FnDef>>> {
        if let Some((module_id, fn_id)) = self.qualified_lookup(id) {
            return self
                .lookup_module(&module_id)
                .and_then(|m| m.lookup_fn(&fn_id));
        }

        match self.lookup_module_for_fn(id).and_then(|m| m.lookup_fn(id)) {
            Some(fn_def) => Some(fn_def),
            None => {
                let state = self.state.borrow();
                match &state.parent {
                    Some(parent_env) => parent_env.lookup_fn(id),
                    None => None,
                }
            }
        }
    }

    fn qualified_lookup(&self, id: &Identifier) -> Option<(Identifier, Identifier)> {
        let parts: Vec<&str> = id.split('.').collect();
        match parts.len() {
            0 => None,
            1 => None,
            len => {
                let (module_id, fn_id) = parts.split_at(len - 1);
                let mid: Identifier = module_id.join(".").into();
                let fid: Identifier = fn_id.join("").into();
                if let Some(mid) = self.lookup_module_alias(&mid) {
                    return Some((mid, fid));
                }
                Some((mid, fid))
            }
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn lookup_module_for_fn(&self, id: &Identifier) -> Option<Module> {
        let curr_mod = self.current_module();
        match curr_mod.lookup_fn(id) {
            Some(_) => Some(curr_mod),
            None => {
                let state = self.state.borrow();
                for (module_id, imports) in state.imported_modules.iter() {
                    match imports {
                        Imports::All => match self.lookup_module(module_id) {
                            Some(module) => match module.lookup_fn(id) {
                                Some(_) => return Some(module),
                                None => continue,
                            },
                            None => continue,
                        },
                        Imports::Specific(imported_ids) => {
                            for iid in imported_ids.iter() {
                                if import_id_matches(iid, id) {
                                    let module =
                                        self.lookup_module(module_id).unwrap_or_else(|| {
                                            panic!(
                                                "module should exist: {} for fn: {}",
                                                module_id, id
                                            )
                                        });
                                    return Some(module);
                                }
                            }
                        }
                    }
                }

                None
            }
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn lookup_module_for_type(&self, id: &Identifier) -> Option<Module> {
        let curr_mod = self.current_module();
        match curr_mod.lookup_type(id) {
            Some(_) => Some(curr_mod),
            None => {
                let state = self.state.borrow();
                for (module_id, imports) in state.imported_modules.iter() {
                    match imports {
                        Imports::All => match self.lookup_module(module_id) {
                            Some(module) => match module.lookup_type(id) {
                                Some(_) => return Some(module),
                                None => continue,
                            },
                            None => continue,
                        },
                        Imports::Specific(imported_ids) => {
                            for iid in imported_ids.iter() {
                                if import_id_matches(iid, id) {
                                    let module =
                                        self.lookup_module(module_id).unwrap_or_else(|| {
                                            panic!(
                                                "module should exist: {} for type: {}",
                                                module_id, id
                                            )
                                        });
                                    return Some(module);
                                }
                            }
                        }
                    }
                }

                None
            }
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn lookup_module_for_const(&self, id: &Identifier) -> Option<Module> {
        let curr_mod = self.current_module();
        match curr_mod.lookup_type(id) {
            Some(_) => Some(curr_mod),
            None => {
                let state = self.state.borrow();
                for (module_id, imports) in state.imported_modules.iter() {
                    match imports {
                        Imports::All => match self.lookup_module(module_id) {
                            Some(module) => match module.lookup_const(id) {
                                Some(_) => return Some(module),
                                None => continue,
                            },
                            None => continue,
                        },
                        Imports::Specific(imported_ids) => {
                            for iid in imported_ids.iter() {
                                if import_id_matches(iid, id) {
                                    let module =
                                        self.lookup_module(module_id).unwrap_or_else(|| {
                                            panic!(
                                                "module should exist: {} for const: {}",
                                                module_id, id
                                            )
                                        });
                                    return Some(module);
                                }
                            }
                        }
                    }
                }

                None
            }
        }
    }
}

fn import_id_matches(imported_id: &ImportedIdentifier, id: &Identifier) -> bool {
    match imported_id {
        ImportedIdentifier::Func(func_id) => func_id == id,
        ImportedIdentifier::Type(type_id) => type_id == id,
        ImportedIdentifier::AliasedFunc(_import_id, alias_id) => alias_id == id,
        ImportedIdentifier::AliasedType(_import_id, alias_id) => alias_id == id,
    }
}
