use std::{collections::HashMap, sync::RwLock};

use sexpr_ir::gast::{symbol::Symbol, Handle};

use crate::value::Value;


#[derive(Debug, Clone)]
pub struct SimpleScope(pub Handle<RwLock<HashMap<Handle<Symbol>, Value>>>);


#[derive(Debug, Clone)]
pub struct Scope {
    pub this_level: SimpleScope,
    pub parent: Option<Handle<Scope>>,
}


impl SimpleScope {
    pub fn new() -> SimpleScope {
        SimpleScope(Handle::new(RwLock::new(HashMap::new())))
    }
}

impl From<HashMap<Handle<Symbol>, Value>> for SimpleScope {
    fn from(i: HashMap<Handle<Symbol>, Value>) -> Self {
        SimpleScope(Handle::new(RwLock::new(i)))
    }
}


impl Scope {
    pub fn new() -> Handle<Scope> {
        let r = Scope {
            this_level: SimpleScope::new(),
            parent: None,
        };
        Handle::new(r)
    }

    pub fn from(this_level: SimpleScope) -> Handle<Scope> {
        let r = Scope {
            this_level,
            parent: None,
        };
        Handle::new(r)
    }

    pub fn new_level(self: &Handle<Scope>, this_level: SimpleScope) -> Handle<Scope> {
        let r = Scope {
            this_level,
            parent: Some(self.clone()),
        };
        Handle::new(r)
    }

    pub fn find_from_raw(&self, k: &str) -> Option<Value> {
        self.find(&Handle::new(Symbol::new(k)))
    }

    pub fn find(&self, k: &Handle<Symbol>) -> Option<Value> {
        let record = self.this_level.0.read().unwrap();
        if let Some(r) = record.get(k) {
            Some(r.clone())
        } else {
            self.parent.clone().map_or_else(|| None, |x| x.find(k))
        }
    }

    pub fn flatten(&self) -> SimpleScope {
        if let Some(p) = &self.parent {
            let p = p.flatten();
            let mut record = p.0.write().unwrap();
            let this_level = self.this_level.0.read().unwrap();
            for (k, v) in this_level.iter() {
                record.insert(k.clone(), v.clone());
            }
            p.clone()
        } else {
            self.this_level.clone()
        }
    }

}
