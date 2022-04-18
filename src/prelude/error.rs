use crate::value::Value;
use crate::value::result::{CResult, CError};

pub(crate) fn native_error(args: Vec<Value>) -> CResult {
    let e = args.get(0).map_or_else(
        || CError::RuntimeError(None),
        |v| CError::RuntimeError(Some(v.clone())));
    Err(e)
}

pub(crate) fn native_unreachable(args: Vec<Value>) -> CResult {
    let e = args.get(0).map_or_else(
        || CError::RuntimeError(None),
        |v| CError::Unreachable(Some(v.clone())));
    Err(e)
}
