pub mod callable;
pub mod result;
pub mod scope;

use std::collections::HashMap;

use sexpr_ir::gast::{Handle, symbol::Symbol};
use callable::Callable;


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
