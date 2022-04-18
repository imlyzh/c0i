use sexpr_ir::gast::Handle;

use crate::value::Value;
use crate::value::result::{CResult, CError};


pub(crate) fn native_add_str(args: Vec<Value>) -> CResult {
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

pub(crate) fn to_str(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let r = args.get(0).unwrap().to_string();
    Ok(Value::Str(Handle::new(r)))
}
