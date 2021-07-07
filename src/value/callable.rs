use std::collections::HashMap;

use sexpr_ir::gast::{symbol::Symbol, Handle};

use super::result::{CError, CResult};
use super::scope::{Scope, SimpleScope};
use crate::ast::Function;
use crate::evaluation::Eval;

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
pub trait Call {
    fn call(&self, args: &[Value]) -> CResult;
}

impl Call for Callable {
    fn call(&self, args: &[Value]) -> CResult {
        match self {
            Callable::Closure(x) => x.call(args),
            Callable::Native(x) => x.call(args),
        }
    }
}

impl Function {
    fn match_args(&self, args: &[Value]) -> Option<SimpleScope> {
        if args.len() == self.prarms.len() {
            let record: HashMap<Handle<Symbol>, Value> = self.prarms
            .iter()
            .zip(args.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
            Some(SimpleScope::from(record))
        } else if self.extend_prarms.is_some() && args.len() > self.prarms.len() {
            let mut record: HashMap<Handle<Symbol>, Value> = self.prarms
            .iter()
            .zip(args[..self.prarms.len()].iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
            record.insert(
                self.extend_prarms.clone().unwrap(),
                Value::from(&args[self.prarms.len()..]));
            Some(SimpleScope::from(record))
        } else {
            None
        }
    }
}

impl Call for Closure {
    fn call(&self, args: &[Value]) -> CResult {
        let Closure(f, env) = self;
        let args_dict = f
        .match_args(&args)
        .ok_or_else(|| CError::PrarmsIsNotMatching(args.to_vec()))?;

        let scope = if let Some(env) = env {
            env.new_level(args_dict)
        } else {
            Scope::from(args_dict)
        };

        if f.bodys.is_empty() {
            Ok(Value::Nil)
        } else if f.bodys.len() == 1 {
            f.bodys.first().unwrap().eval(&scope)
        } else {
            let body_end = f.bodys.last().unwrap();
            let bodys = &f.bodys[..f.bodys.len()-1];
            for i in bodys {
                i.eval(&scope)?;
            }
            body_end.eval(&scope)
        }
    }
}

impl Call for NativeFunction {
    fn call(&self, i: &[Value]) -> CResult {
        (self.interface)(i.to_vec())
    }
}
