use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::{Dict, Value};
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use super::LOCATION;


fn make_dict(args: Vec<Value>) -> CResult {
    if args.len() != 0 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    Ok(Value::Dict(Dict::default()))
}

impl_wrap!(MAKE_DICT, MAKE_DICT_NAME, make_dict, "make-dict", &LOCATION);
