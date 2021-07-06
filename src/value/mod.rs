pub mod callable;
pub mod result;
pub mod scope;

use std::collections::HashMap;

use callable::Callable;
use sexpr_ir::gast::{symbol::Symbol, Handle};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Char(char),
    Uint(u64),
    Int(i64),
    Float(f64),
    Str(Handle<String>),
    Sym(Handle<Symbol>),
    Pair(Handle<Pair>),
    Dict(Handle<Dict>),
    Vec(Handle<Vector>),
    Callable(Callable),
}

#[derive(Debug, Clone)]
pub struct Pair(pub Value, pub Value);

#[derive(Debug, Clone)]
pub struct Dict(pub HashMap<Handle<String>, Value>);

#[derive(Debug, Clone)]
pub struct Vector(pub Vec<Value>);


impl From<&[Value]> for Value {
    fn from(i: &[Value]) -> Self {
        if let Some(left) = i.first() {
            let right = Value::from(&i[1..]);
            Value::Pair(Handle::new(Pair(left.clone(), right)))
        } else {
            Value::Nil
        }
    }
}