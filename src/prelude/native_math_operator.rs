use crate::value::Value;
use crate::value::result::{CResult, CError};


macro_rules! impl_native_num_add {
    ($fn_name:ident, $ctor:ident, $init:expr) => (
        pub(crate) fn $fn_name(args: Vec<Value>) -> CResult {
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

macro_rules! impl_native_num_sub {
    ($fn_name:ident, $ctor:ident) => {
        pub(crate) fn $fn_name(args: Vec<Value>) -> CResult {
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

macro_rules! impl_native_num_mul {
    ($fn_name:ident, $ctor:ident, $init:expr) => (
        pub(crate) fn $fn_name(args: Vec<Value>) -> CResult {
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

macro_rules! impl_native_num_div {
    ($fn_name:ident, $ctor:ident) => {
        pub(crate) fn $fn_name(args: Vec<Value>) -> CResult {
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

pub(crate) fn native_div_float(args: Vec<Value>) -> CResult {
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
