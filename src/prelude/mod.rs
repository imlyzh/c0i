pub mod error;
pub mod raw_operator;
pub mod native_eq_ord_operator;
pub mod dynamic_type_check;
pub mod native_math_operator;
pub mod native_bool_operator;
pub mod native_string_operator;
pub mod native_dict_operator;

use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Location};

use crate::value::{Value, callable::Callable, scope::Scope};

use error::*;
use raw_operator::*;
use native_eq_ord_operator::*;
use dynamic_type_check::*;
use native_math_operator::*;
use native_string_operator::*;
use native_bool_operator::*;
use native_dict_operator::*;


lazy_static! {
    static ref LOCATION: Location = Location {
        path: Handle::new("<builtin>".to_string()),
        line: 0,
        colum: 0,
        pos: 0
    };
}

#[macro_export]
macro_rules! impl_wrap {
    ($id:ident, $symid:ident, $fun:ident, $name:expr, $location:expr) => {
        lazy_static! {
            pub static ref $symid: Handle<Symbol> = Handle::new(Symbol::from($name, $location));
            pub static ref $id: NativeFunction = NativeFunction {
                name: $symid.clone(),
                is_pure: true,
                type_sign: (),
                interface: $fun
            };
        }
    };
}

macro_rules! set_wrap {
    ($rcd:expr, $name:ident, $fun:ident) => {
        $rcd.insert(
            $name.clone(),
            Value::Callable(Callable::Native(Handle::new($fun.clone()))));
    };
}


pub fn init() -> Handle<Scope> {
    let record = Scope::new();
    {
        let mut rcd = record.this_level.0.write().unwrap();
        // error
        set_wrap!(rcd, ERROR_NAME, ERROR_WRAP);
        set_wrap!(rcd, UNRECHABLE_NAME, UNRECHABLE_WRAP);
        // read
        set_wrap!(rcd, READ_NAME, READ_WRAP);
        // list
        set_wrap!(rcd, CAR_NAME, CAR_WRAP);
        set_wrap!(rcd, CDR_NAME, CDR_WRAP);
        set_wrap!(rcd, CONS_NAME, CONS_WRAP);
        set_wrap!(rcd, VECTOR_NAME, VECTOR_WRAP);
        //dict
        set_wrap!(rcd, MAKE_DICT_NAME, MAKE_DICT);
        // set_wrap!(rcd, VECTOR_MAP_NAME, VECTOR_MAP_WRAP);
        set_wrap!(rcd, VECTOR_REDUCE_NAME, VECTOR_REDUCE_WRAP);
        set_wrap!(rcd, IGNORE_NAME, IGNORE_WRAP);
        set_wrap!(rcd, ID_NAME, ID_WRAP);
        // to_literal
        set_wrap!(rcd, LITERAL_NAME, LITERAL_WRAP);
        // eq ord
        set_wrap!(rcd, EQ_NAME, EQ_WRAP);
        set_wrap!(rcd, NE_NAME, NE_WRAP);
        set_wrap!(rcd, LT_NAME, LT_WRAP);
        set_wrap!(rcd, GT_NAME, GT_WRAP);
        set_wrap!(rcd, LE_NAME, LE_WRAP);
        set_wrap!(rcd, GE_NAME, GE_WRAP);
        // bool
        set_wrap!(rcd, BOOL_NOT_NAME, BOOL_NOT_WRAP);
        set_wrap!(rcd, BOOL_AND_NAME, BOOL_AND_WRAP);
        set_wrap!(rcd, BOOL_OR_NAME, BOOL_OR_WRAP);
        // math
        // add
        set_wrap!(rcd, ADD_INT_NAME, ADD_INT_WRAP);
        set_wrap!(rcd, ADD_UINT_NAME, ADD_UINT_WRAP);
        set_wrap!(rcd, ADD_FLOAT_NAME, ADD_FLOAT_WRAP);
        // sub
        set_wrap!(rcd, SUB_INT_NAME, SUB_INT_WRAP);
        set_wrap!(rcd, SUB_UINT_NAME, SUB_UINT_WRAP);
        set_wrap!(rcd, SUB_FLOAT_NAME, SUB_FLOAT_WRAP);
        // mul
        set_wrap!(rcd, MUL_INT_NAME, MUL_INT_WRAP);
        set_wrap!(rcd, MUL_UINT_NAME, MUL_UINT_WRAP);
        set_wrap!(rcd, MUL_FLOAT_NAME, MUL_FLOAT_WRAP);
        // div
        set_wrap!(rcd, DIV_INT_NAME, DIV_INT_WRAP);
        set_wrap!(rcd, DIV_UINT_NAME, DIV_UINT_WRAP);
        set_wrap!(rcd, DIV_FLOAT_NAME, DIV_FLOAT_WRAP);

        // type check
        set_wrap!(rcd, IS_NIL_NAME, IS_NIL_WRAP);
        set_wrap!(rcd, IS_CHAR_NAME, IS_CHAR_WRAP);
        set_wrap!(rcd, IS_BOOL_NAME, IS_BOOL_WRAP);
        set_wrap!(rcd, IS_INT_NAME, IS_INT_WRAP);
        set_wrap!(rcd, IS_UINT_NAME, IS_UINT_WRAP);
        set_wrap!(rcd, IS_FLOAT_NAME, IS_FLOAT_WRAP);
        set_wrap!(rcd, IS_STR_NAME, IS_STR_WRAP);
        set_wrap!(rcd, IS_SYM_NAME, IS_SYM_WRAP);
        set_wrap!(rcd, IS_PAIR_NAME, IS_PAIR_WRAP);
        set_wrap!(rcd, IS_DICT_NAME, IS_DICT_WRAP);
        set_wrap!(rcd, IS_VEC_NAME, IS_VEC_WRAP);
        set_wrap!(rcd, IS_CALLABLE_NAME, IS_CALLABLE_WRAP);
    }
    record
}