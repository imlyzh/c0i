pub mod raw_operator;

use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Location};

use crate::value::{Value, callable::Callable, scope::Scope};

use raw_operator::*;


lazy_static! {
    static ref LOCATION: Location = Location {
        path: Handle::new("<builtin>".to_string()),
        line: 0,
        colum: 0,
        pos: 0
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
        set_wrap!(rcd, CAR_NAME, CAR_WRAP);
        set_wrap!(rcd, CDR_NAME, CDR_WRAP);
        set_wrap!(rcd, CONS_NAME, CONS_WRAP);
        set_wrap!(rcd, VECTOR_NAME, VECTOR_WRAP);
        set_wrap!(rcd, VECTOR_MAP_NAME, VECTOR_MAP_WRAP);
        set_wrap!(rcd, VECTOR_REDUCE_NAME, VECTOR_REDUCE_WRAP);
        set_wrap!(rcd, IGNORE_NAME, IGNORE_WRAP);
        set_wrap!(rcd, ID_NAME, ID_WRAP);
    }
    record
}