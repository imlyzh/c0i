use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use super::LOCATION;


macro_rules! impl_native_num_add {
    ($fn_name:ident, $ctor:ident, $init:expr) => (
        fn $fn_name(args: Vec<Value>) -> CResult {
            let mut ret = $init;
            for arg in args.into_iter() {
                if let Value::$ctor(v) = arg {
                    ret += v;
                } else {
                    return CResult::Err(CError::TypeError((), arg));
                }
            }
            CResult::Ok(Value::$ctor(ret))
        }
    )
}

impl_native_num_add! { native_add_int, Int, 0 }
impl_native_num_add! { native_add_uint, Uint, 0 }
impl_native_num_add! { native_add_float, Float, 0.0 }

impl_wrap!(ADD_INT_WRAP, ADD_INT_NAME, native_add_int, "+i", &LOCATION);
impl_wrap!(ADD_UINT_WRAP, ADD_UINT_NAME, native_add_uint, "+u", &LOCATION);
impl_wrap!(ADD_FLOAT_WRAP, ADD_FLOAT_NAME, native_add_float, "+f", &LOCATION);


/*
fn native_add(args: Vec<Value>) -> CResult {
    if args.len() <= 1 {
        return CResult::Err(CError::PrarmsIsNotMatching(args));
    }

    let v = &args[0];
    match v {
        Value::Int(_) => native_add_int(args),
        Value::Float(_) => native_add_float(args),
        Value::Str(_) => native_add_str(args),
        _ => CResult::Err(CError())
    }
}
*/

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
                    return CResult::Err(CError::TypeError((), arg));
                }
            }

            return CResult::Ok(Value::$ctor(ret));
        }
    }
}

impl_native_num_sub! { native_sub_int, Int }
impl_native_num_sub! { native_sub_uint, Uint }
impl_native_num_sub! { native_sub_float, Float }

impl_wrap!(SUB_INT_WRAP, SUB_INT_NAME, native_sub_int, "-i", &LOCATION);
impl_wrap!(SUB_UINT_WRAP, SUB_UINT_NAME, native_sub_uint, "-u", &LOCATION);
impl_wrap!(SUB_FLOAT_WRAP, SUB_FLOAT_NAME, native_sub_float, "-f", &LOCATION);

/*
fn native_sub(args: Vec<Value>) -> CResult {
    if args.len() <= 1 {
        return CResult::Err(CError::PrarmsIsNotMatching(args));
    }

    match &args[0] {
        Value::Int(_) => native_sub_int(args),
        Value::Float(_) => native_sub_float(args),
        _ => CResult::Err(CError())
    }
}
 */

macro_rules! impl_native_num_mul {
    ($fn_name:ident, $ctor:ident, $init:expr) => (
        fn $fn_name(args: Vec<Value>) -> CResult {
            let mut ret = $init;
            for arg in args.into_iter() {
                if let Value::$ctor(v) = arg {
                    ret *= v;
                } else {
                    return CResult::Err(CError::TypeError((), arg));
                }
            }
            CResult::Ok(Value::$ctor(ret))
        }
    )
}

impl_native_num_mul! { native_mul_int, Int, 1 }
impl_native_num_mul! { native_mul_uint, Uint, 1 }
impl_native_num_mul! { native_mul_float, Float, 1.0 }

impl_wrap!(MUL_INT_WRAP, MUL_INT_NAME, native_mul_int, "*i", &LOCATION);
impl_wrap!(MUL_UINT_WRAP, MUL_UINT_NAME, native_mul_uint, "*u", &LOCATION);
impl_wrap!(MUL_FLOAT_WRAP, MUL_FLOAT_NAME, native_mul_float, "*f", &LOCATION);

/*
fn native_mul(args: Vec<Value>) -> CResult {
    if args.len() <= 1 {
        return CResult::Err(CError::PrarmsIsNotMatching(args));
    }

    match &args[0] {
        Value::Int(_) => native_mul_int(args),
        Value::Float(_) => native_mul_float(args),
        _ => CResult::Err(CError())
    }
}
 */

macro_rules! impl_native_num_div {
    ($fn_name:ident, $ctor:ident) => {
        fn $fn_name(args: Vec<Value>) -> CResult {
            let mut ret = if let Value::$ctor(v) = &args[0] {
                *v
            } else {
                panic!()
            };

            for arg in args.into_iter().skip(1) {
                if let Value::$ctor(v) = arg {
                    ret = ret.checked_div(v).ok_or(CError::ZeroDivisionError)?;
                } else {
                    return CResult::Err(CError::TypeError((), arg));
                }
            }

            Ok(Value::$ctor(ret))
        }
    }
}

impl_native_num_div! { native_div_int, Int }
impl_native_num_div! { native_div_uint, Uint }

impl_wrap!(DIV_INT_WRAP, DIV_INT_NAME, native_div_int, "/i", &LOCATION);
impl_wrap!(DIV_UINT_WRAP, DIV_UINT_NAME, native_div_uint, "/u", &LOCATION);


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
            return CResult::Err(CError::TypeError((), arg));
        }
    }

    Ok(Value::Float(ret))
}

impl_wrap!(DIV_FLOAT_WRAP, DIV_FLOAT_NAME, native_div_float, "/f", &LOCATION);
