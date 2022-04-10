use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use crate::value::Pair;

use super::LOCATION;


impl Value {
    #[allow(dead_code)]
    fn to_literal(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(v) => if *v {write!(f, "true")} else {write!(f, "false")},
            Value::Int(v) => if *v > 0 {write!(f, "+{}", v)} else {write!(f, "{}", v)},
            Value::Uint(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Sym(v) => write!(f, "{}", v.0),
            Value::Str(v) => write!(f, "\"{}\"", v), // todo: escape
            Value::Char(v) => write!(f, "(char \"{}\")", v), // todo: escape
            Value::Pair(v) => v.to_literal(f),
            Value::Dict(_) => panic!("error: to_literal is not supported for dict"),
            Value::Vec(_) => panic!("error: to_literal is not supported for vec"),
            Value::Callable(_) => panic!("error: to_literal is not supported for callable"),
        }
    }
}

impl Pair {
    fn to_literal(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut this = self;
        write!(f, "(")?;
        let mut start = true;
        loop {
            match this {
                Pair(v, Value::Pair(t)) => {
                    if !start {
                        write!(f, " ")?;
                    }
                    v.to_literal(f)?;
                    this = t;
                    start = false;
                    continue;
                },
                Pair(v, Value::Nil) => {
                    if !start {
                        write!(f, " ")?;
                    }
                    v.to_literal(f)?;
                    break;
                },
                Pair(v, t) => {
                    if !start {
                        write!(f, " ")?;
                    }
                    v.to_literal(f)?;
                    write!(f, " . ")?;
                    t.to_literal(f)?;
                    break;
                },
            }
        }
        write!(f, ")")
    }
}

fn literal(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let r = args.get(0).unwrap().to_string();
    Ok(Value::Str(Handle::new(r)))
}

impl_wrap!(LITERAL_WRAP, LITERAL_NAME, literal, "literal", &LOCATION);
