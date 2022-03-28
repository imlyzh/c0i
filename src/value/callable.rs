use std::collections::HashMap;
use std::fmt::Display;

use sexpr_ir::gast::{symbol::Symbol, Handle};

use super::result::{CError, CResult};
use super::scope::{Scope, SimpleScope};
use crate::ast::Function;

use super::Value;

#[derive(Debug, Clone)]
pub struct Closure(pub Function, pub Option<Handle<Scope>>);

type NativeInterface = fn(Vec<Value>) -> CResult;

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub name: Handle<Symbol>,
    pub is_pure: bool,
    pub type_sign: (),
    pub interface: NativeInterface,
}

#[derive(Debug, Clone)]
pub enum Callable {
    Closure(Handle<Closure>),
    Native(Handle<NativeFunction>),
}

impl PartialEq for Callable {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}


impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Closure(c) => if let Some(n) = c.0.name.clone() {
                write!(f, "<function '{} in \"{}:{}:{}\">", n, c.0.pos.path, c.0.pos.line, c.0.pos.colum)
            } else {
                write!(f, "<lambda in \"{}:{}:{}\">", c.0.pos.path, c.0.pos.line, c.0.pos.colum)
            },
            Callable::Native(c) => write!(f, "<native '{}>", c.name),
        }
    }
}


impl Function {
    pub fn match_args(&self, args: &[Value]) -> Result<SimpleScope, CError> {
        if self.extend_params.is_some() && args.len() >= self.params.len() {
            let mut record: HashMap<Handle<Symbol>, Value> = self.params
            .iter()
            .zip(args[..self.params.len()].iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
            record.insert(
                self.extend_params.clone().unwrap(),
                Value::from(&args[self.params.len()..]));
            Ok(SimpleScope::from(record))
        } else if args.len() == self.params.len() {
            let record: HashMap<Handle<Symbol>, Value> = self.params
            .iter()
            .zip(args.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
            Ok(SimpleScope::from(record))
        } else {
            Err(CError::ArgsNotMatching(self.params.len(), args.len()))
        }
    }
}
