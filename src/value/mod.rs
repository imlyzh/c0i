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


macro_rules! impl_is_type {
    ($name:ident, $tp:ident) => {
        pub fn $name(&self) -> bool {
            if let Value::$tp(_) = self {
                true
            } else {
                false
            }
        }
    };
}


impl Value {
    pub fn is_nil(&self) -> bool {
        if let Value::Nil = self {
            true
        } else {
            false
        }
    }
    impl_is_type!(is_bool, Bool);
    impl_is_type!(is_char, Char);
    impl_is_type!(is_int, Int);
    impl_is_type!(is_uint, Uint);
    impl_is_type!(is_float, Float);
    impl_is_type!(is_str, Str);
    impl_is_type!(is_sym, Sym);
    impl_is_type!(is_pair, Pair);
    impl_is_type!(is_dict, Dict);
    impl_is_type!(is_vec, Vec);
    impl_is_type!(is_callable, Callable);
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