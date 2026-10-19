use crate::ast::Expr;
use crate::value::Value;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct FunctionDef {
    pub params: Vec<String>,
    pub param_types: Vec<Option<String>>,
    pub body: Vec<Expr>,
}

pub struct Environment {
    pub vars: FxHashMap<String, Value>,
    pub funcs: FxHashMap<String, FunctionDef>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: FxHashMap::default(),
            funcs: FxHashMap::default(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            vars: FxHashMap::default(),
            funcs: FxHashMap::default(),
            parent: Some(parent),
        }
    }

    pub fn lookup_var(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.vars.get(name) {
            return Some(val.clone());
        }
        if let Some(ref p) = self.parent {
            return p.borrow().lookup_var(name);
        }
        None
    }

    pub fn set_var(&mut self, name: String, val: Value) {
        self.vars.insert(name, val);
    }

    pub fn lookup_func(&self, name: &str) -> Option<FunctionDef> {
        if let Some(f) = self.funcs.get(name) {
            return Some(f.clone());
        }
        if let Some(ref p) = self.parent {
            return p.borrow().lookup_func(name);
        }
        None
    }

    pub fn set_func(&mut self, name: String, f: FunctionDef) {
        self.funcs.insert(name, f);
    }

    pub fn clear(&mut self) {
        self.vars.clear();
        self.funcs.clear();
        self.parent = None;
    }
}
