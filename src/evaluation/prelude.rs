use sexpr_ir::gast::Handle;

use crate::value::Value;
use crate::value::result::{CResult, CError};

macro_rules! impl_native_num_add {
    ($fn_name:ident, $ctor:ident, $init:expr) => (
        fn $fn_name(args: Vec<Value>) -> CResult {
            let mut ret = $init;
            for arg in args.into_iter() {
                if let Value::$ctor(v) = arg {
                    ret += v;
                } else {
                    return CResult::Err(CError());
                }
            }
            CResult::Ok(Value::$ctor(ret))
        }
    )
}

impl_native_num_add! { native_add_int, Int, 0 }
impl_native_num_add! { native_add_float, Float, 0.0 }

fn native_add_str(args: Vec<Value>) -> CResult {
    let mut ret = String::new();
    for arg in args.into_iter() {
        if let Value::Str(v) = arg {
            ret += &*v;
        } else {
            return CResult::Err(CError());
        }
    }
    CResult::Ok(Value::Str(Handle::new(ret)))
}

fn native_add(args: Vec<Value>) -> CResult {
    if args.len() <= 1 {
        return CResult::Err(CError());
    }
    
    match &args[0] {
        Value::Int(_) => native_add_int(args),
        Value::Float(_) => native_add_float(args),
        Value::Str(_) => native_add_str(args),
        _ => CResult::Err(CError())
    }
}

macro_rules! impl_native_num_sub {
    ($fn_name:ident, $ctor:ident) => {
        fn $fn_name(args: Vec<Value>) -> CResult {
            let mut ret = if let Value::$ctor(v) = &args[0] {
                *v
            } else {
                panic!()
            };
            
            for arg in args.into_iter().skip(1) {
                if let Value::$ctor(v) = arg {
                    ret -= v;
                } else {
                    return CResult::Err(CError());
                }
            }
            
            return CResult::Ok(Value::$ctor(ret));
        }
    }
}

impl_native_num_sub! { native_sub_int, Int }
impl_native_num_sub! { native_sub_float, Float }

fn native_sub(args: Vec<Value>) -> CResult {
    if args.len() <= 1 {
        return CResult::Err(CError());
    }
    
    match &args[0] {
        Value::Int(_) => native_sub_int(args),
        Value::Float(_) => native_sub_float(args),
        _ => CResult::Err(CError())
    }
}

macro_rules! impl_native_num_mul {
    ($fn_name:ident, $ctor:ident, $init:expr) => (
        fn $fn_name(args: Vec<Value>) -> CResult {
            let mut ret = $init;
            for arg in args.into_iter() {
                if let Value::$ctor(v) = arg {
                    ret *= v;
                } else {
                    return CResult::Err(CError());
                }
            }
            CResult::Ok(Value::$ctor(ret))
        }
    )
}

impl_native_num_mul! { native_mul_int, Int, 1 }
impl_native_num_mul! { native_mul_float, Float, 1.0 }

fn native_mul(args: Vec<Value>) -> CResult {
    if args.len() <= 1 {
        return CResult::Err(CError());
    }
    
    match &args[0] {
        Value::Int(_) => native_mul_int(args),
        Value::Float(_) => native_mul_float(args),
        _ => CResult::Err(CError())
    }
}

fn native_div_int(args: Vec<Value>) -> CResult {
    let mut ret = if let Value::Int(v) = &args[0] {
        *v
    } else {
        panic!()
    };
    
    for arg in args.into_iter().skip(1) {
        if let Value::Int(v) = arg {
            ret = ret.checked_div(v).ok_or(CError())?;
        } else {
            return CResult::Err(CError());
        }
    }
            
    return CResult::Ok(Value::Int(ret));
}

fn native_div_float(args: Vec<Value>) -> CResult {
    let mut ret = if let Value::Float(v) = &args[0] {
        *v
    } else {
        panic!()
    };
    
    for arg in args.into_iter().skip(1) {
        if let Value::Float(v) = arg {
            ret /= v;
        } else {
            return CResult::Err(CError());
        }
    }
    
    return CResult::Ok(Value::Float(ret));
}

fn display_item(v: Value) {
    match v {
        Value::Nil => print!("'()"),
        Value::Int(iValue) => print!("{}", iValue),
        Value::Float(fValue) => print!("{}", fValue),
        Value::Char(chValue) => print!("'{}'", chValue),
        Value::Uint(uValue) => print!("{}", uValue),
        Value::Str(sValue) => print!("\"{}\"", sValue),
        
    }
}
