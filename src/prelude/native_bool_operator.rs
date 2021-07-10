use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use super::LOCATION;


fn native_bool_not(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::PrarmsIsNotMatching(1, args.len()));
    }
    let v = args.get(0).unwrap();
    if let Value::Bool(b) = v {
        Ok(Value::Bool(!b))
    } else {
        Err(CError::TypeError((), v.clone()))
    }
}

impl_wrap!(BOOL_NOT_WRAP, BOOL_NOT_NAME, native_bool_not, "not", &LOCATION);
