use std::collections::HashMap;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::value::Value;


#[derive(Debug, Clone)]
pub struct SimpleScope(pub HashMap<Handle<Symbol>, Value>);


#[derive(Debug, Clone)]
pub struct Scope {
    this_level: Handle<SimpleScope>,
    parent: Option<Handle<Scope>>
}

impl Scope {
    pub fn get_from_raw(&self, k: &str) -> Option<Value> {
        self.get(&Handle::new(Symbol::new(k)))
    }

    pub fn get(&self, k: &Handle<Symbol>) -> Option<Value> {
        let r = self.this_level.0.get(k);
        if let Some(r) = r {
            Some(r.clone())
        } else {
            self.parent.clone().map_or_else(||None, |x| x.get(k))
        }
    }
}
