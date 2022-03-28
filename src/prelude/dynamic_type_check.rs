use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use super::LOCATION;


macro_rules! impl_native_is_type {
    ($name:ident, $fun:ident) => {
        fn $name(args: Vec<Value>) -> CResult {
            if args.len() != 1 {
                return Err(CError::PrarmsIsNotMatching(1, args.len()));
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


impl_wrap!(IS_NIL_WRAP      , IS_NIL_NAME       , native_is_nil     , "null?"        , &LOCATION);
impl_wrap!(IS_CHAR_WRAP     , IS_CHAR_NAME      , native_is_char    , "char?"       , &LOCATION);
impl_wrap!(IS_BOOL_WRAP     , IS_BOOL_NAME      , native_is_bool    , "bool?"       , &LOCATION);
impl_wrap!(IS_INT_WRAP      , IS_INT_NAME       , native_is_int     , "int?"        , &LOCATION);
impl_wrap!(IS_UINT_WRAP     , IS_UINT_NAME      , native_is_uint    , "uint?"       , &LOCATION);
impl_wrap!(IS_FLOAT_WRAP    , IS_FLOAT_NAME     , native_is_float   , "float?"      , &LOCATION);
impl_wrap!(IS_STR_WRAP      , IS_STR_NAME       , native_is_str     , "str?"        , &LOCATION);
impl_wrap!(IS_SYM_WRAP      , IS_SYM_NAME       , native_is_sym     , "sym?"        , &LOCATION);
impl_wrap!(IS_PAIR_WRAP     , IS_PAIR_NAME      , native_is_pair    , "pair?"       , &LOCATION);
impl_wrap!(IS_DICT_WRAP     , IS_DICT_NAME      , native_is_dict    , "dict?"       , &LOCATION);
impl_wrap!(IS_VEC_WRAP      , IS_VEC_NAME       , native_is_vec     , "vec?"        , &LOCATION);
impl_wrap!(IS_CALLABLE_WRAP , IS_CALLABLE_NAME  , native_is_callable, "callable?"   , &LOCATION);
