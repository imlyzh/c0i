use crate::value::{Dict, Value};
use crate::value::result::{CResult, CError};


pub(crate) fn make_dict(args: Vec<Value>) -> CResult {
    if args.len() != 0 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    Ok(Value::Dict(Dict::default()))
}
