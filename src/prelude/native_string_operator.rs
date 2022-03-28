use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use super::LOCATION;


fn native_add_str(args: Vec<Value>) -> CResult {
    let mut ret = String::new();
    for arg in args.into_iter() {
        if let Value::Str(v) = arg {
            ret += &*v;
        } else {
            return CResult::Err(CError::TypeError((), arg));
        }
    }
    CResult::Ok(Value::Str(Handle::new(ret)))
}

impl_wrap!(ADD_STR_WRAP, ADD_STR_NAME, native_add_str, "+s", &LOCATION);


fn to_literal(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let r = args.get(0).unwrap().to_string();
    Ok(Value::Str(Handle::new(r)))
}

impl_wrap!(LITERAL_WRAP, LITERAL_NAME, to_literal, "literal", &LOCATION);
