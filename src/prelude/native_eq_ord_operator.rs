use crate::value::Value;
use crate::value::result::{CResult, CError};


pub(crate) fn eq(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a == b))
}

pub(crate) fn ne(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a != b))
}

pub(crate) fn lt(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a < b))
}

pub(crate) fn gt(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a > b))
}

pub(crate) fn le(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a <= b))
}

pub(crate) fn ge(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a >= b))
}
