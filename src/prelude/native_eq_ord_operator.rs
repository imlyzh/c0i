use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use super::LOCATION;


fn eq(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a == b))
}

impl_wrap!(EQ_WRAP, EQ_NAME, eq, "eq?", &LOCATION);

fn ne(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a != b))
}

impl_wrap!(NE_WRAP, NE_NAME, ne, "ne?", &LOCATION);

fn lt(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a < b))
}

impl_wrap!(LT_WRAP, LT_NAME, lt, "<", &LOCATION);

fn gt(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a > b))
}

impl_wrap!(GT_WRAP, GT_NAME, gt, ">", &LOCATION);

fn le(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a <= b))
}

impl_wrap!(LE_WRAP, LE_NAME, le, "<=", &LOCATION);

fn ge(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(2, args.len()));
    }
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    Ok(Value::Bool(a >= b))
}

impl_wrap!(GE_WRAP, GE_NAME, ge, ">=", &LOCATION);
