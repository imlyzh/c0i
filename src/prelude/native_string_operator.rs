use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::{Callable, NativeFunction};
use crate::value::result::{CResult, CError};

use super::LOCATION;


fn native_add_str(args: Vec<Value>) -> CResult {
    let mut ret = String::new();
    for arg in args.into_iter() {
        if let Value::Str(v) = arg {
            ret += &*v;
        } else {
            return CResult::Err(CError::TypeError(((), arg)));
        }
    }
    CResult::Ok(Value::Str(Handle::new(ret)))
}

impl_wrap!(ADD_STR_WRAP, ADD_STR_NAME, native_add_str, "+s", &LOCATION);


pub fn display_item(v: Value) {
    match v {
        Value::Nil => print!("'()"),
        Value::Int(v) => print!("{}", v),
        Value::Float(v) => print!("{}", v),
        Value::Char(v) => print!("'{}'", v),
        Value::Uint(v) => print!("{}", v),
        Value::Str(v) => print!("\"{}\"", v),
        Value::Bool(v) => print!("\"{}\"", v),
        Value::Sym(v) => print!("'\"{}\"", v),
        Value::Pair(_) => todo!(),
        Value::Dict(_) => todo!(),
        Value::Vec(_) => todo!(),
        Value::Callable(v) => match v {
            Callable::Closure(c) => if let Some(n) = c.0.name.clone() {
                print!("<function {}>", n);
            } else {
                print!("<lambda>");
            },
            Callable::Native(c) => print!("<native {}>", c.name),
        },
    }
}

fn native_display(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(args));
    }
    display_item(args[0].clone());
    Ok(Value::Nil)
}

impl_wrap!(DISPLAY_WRAP, DISPLAY_NAME, native_display, "display", &LOCATION);