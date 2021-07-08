pub mod raw_operator;
pub mod dynamic_type_check;
pub mod native_math_operator;
pub mod native_string_operator;

use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Location};

use crate::value::{Value, callable::Callable, scope::Scope};

use raw_operator::*;
use dynamic_type_check::*;
use native_math_operator::*;
use native_string_operator::*;


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
        // list
        set_wrap!(rcd, CAR_NAME, CAR_WRAP);
        set_wrap!(rcd, CDR_NAME, CDR_WRAP);
        set_wrap!(rcd, CONS_NAME, CONS_WRAP);
        set_wrap!(rcd, VECTOR_NAME, VECTOR_WRAP);
        set_wrap!(rcd, VECTOR_MAP_NAME, VECTOR_MAP_WRAP);
        set_wrap!(rcd, VECTOR_REDUCE_NAME, VECTOR_REDUCE_WRAP);
        set_wrap!(rcd, IGNORE_NAME, IGNORE_WRAP);
        set_wrap!(rcd, ID_NAME, ID_WRAP);
        // display
        set_wrap!(rcd, DISPLAY_NAME, DISPLAY_WRAP);
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