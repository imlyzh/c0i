use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use super::LOCATION;


fn native_error(args: Vec<Value>) -> CResult {
    let e = args.get(0).map_or_else(
        || CError::RuntimeError(None),
        |v| CError::RuntimeError(Some(v.clone())));
    Err(e)
}

impl_wrap!(ERROR_WRAP, ERROR_NAME, native_error, "error", &LOCATION);


fn native_unreachable(args: Vec<Value>) -> CResult {
    let e = args.get(0).map_or_else(
        || CError::RuntimeError(None),
        |v| CError::Unreachable(Some(v.clone())));
    Err(e)
}

impl_wrap!(UNRECHABLE_WRAP, UNRECHABLE_NAME, native_unreachable, "unreachable", &LOCATION);
