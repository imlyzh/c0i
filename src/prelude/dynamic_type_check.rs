use crate::value::Value;
use crate::value::result::{CResult, CError};

macro_rules! impl_native_is_type {
    ($name:ident, $fun:ident) => {
        pub(crate) fn $name(args: Vec<Value>) -> CResult {
            if args.len() != 1 {
                return Err(CError::ArgsNotMatching(1, args.len()));
            }
            Ok(Value::Bool(args.get(0).unwrap().$fun()))
        }
    };
}

impl_native_is_type!(native_is_nil, is_nil);
impl_native_is_type!(native_is_char, is_char);
impl_native_is_type!(native_is_bool, is_bool);
impl_native_is_type!(native_is_int, is_int);
impl_native_is_type!(native_is_uint, is_uint);
impl_native_is_type!(native_is_float, is_float);
impl_native_is_type!(native_is_str, is_str);
impl_native_is_type!(native_is_sym, is_sym);
impl_native_is_type!(native_is_pair, is_pair);
impl_native_is_type!(native_is_dict, is_dict);
impl_native_is_type!(native_is_vec, is_vec);
impl_native_is_type!(native_is_callable, is_callable);
