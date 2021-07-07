use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::value::{Pair, Value, Vector, callable::{Call, NativeFunction}, result::{CError, CResult}};

use super::LOCATION;


macro_rules! impl_wrap {
    ($id:ident, $symid:ident, $fun:ident, $name:expr) => {
        lazy_static! {
            pub static ref $symid: Handle<Symbol> = Handle::new(Symbol::from($name, &LOCATION));
            pub static ref $id: NativeFunction = NativeFunction {
                name: $symid.clone(),
                is_pure: true,
                type_sign: (),
                interface: $fun
            };
        }
    };
}


fn car(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::PrarmsIsNotMatching(args));
    }
    let value = args.get(0).unwrap();
    if let Value::Pair(pair) = value {
        Ok(pair.0.clone())
    } else {
        Err(CError::TypeError(((), value.clone())))
    }
}

impl_wrap!(CAR_WRAP, CAR_NAME, car, "car");


fn cdr(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::PrarmsIsNotMatching(args));
    }
    let value = args.get(0).unwrap();
    if let Value::Pair(pair) = value {
        Ok(pair.1.clone())
    } else {
        Err(CError::TypeError(((), value.clone())))
    }
}

impl_wrap!(CDR_WRAP, CDR_NAME, cdr, "cdr");


fn cons(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(args));
    }
    Ok(Value::Pair(Handle::new(Pair(
            args.get(0).unwrap().clone(),
            args.get(1).unwrap().clone()))))
}

impl_wrap!(CONS_WRAP, CONS_NAME, cons, "cons");


fn vector(args: Vec<Value>) -> CResult {
    Ok(Value::Vec(Handle::new(Vector(args))))
}

impl_wrap!(VECTOR_WRAP, VECTOR_NAME, vector, "vector");


fn vector_map(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(args));
    }
    let vector = args.get(1).unwrap();
    let callable = args.get(0).unwrap();
    let vector = if let Value::Vec(vec) = vector {
        vec
    } else {
        return Err(CError::TypeError(((), vector.clone())));
    };
    let callable = if let Value::Callable(callable) = callable {
        callable
    } else {
        return Err(CError::TypeError(((), callable.clone())));
    };
    let r: Result<Vec<_>, _> = vector.0
    .iter()
    .map(|x| callable.call(&[x.clone()]))
    .collect();
    Ok(Value::Vec(Handle::new(Vector(r?))))
}

impl_wrap!(VECTOR_MAP_WRAP, VECTOR_MAP_NAME, vector_map, "vector-map");


fn vector_reduce(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(args));
    }
    let vector = args.get(1).unwrap();
    let callable = args.get(0).unwrap();
    let vector = if let Value::Vec(vec) = vector {
        vec
    } else {
        return Err(CError::TypeError(((), vector.clone())));
    };
    let callable = if let Value::Callable(callable) = callable {
        callable
    } else {
        return Err(CError::TypeError(((), callable.clone())));
    };
    let mut iter = vector.0.iter();
    let init = iter.next().map_or(Value::Nil, Value::clone);
    iter.try_fold(init, |x, y| callable.call(&[x, y.clone()]))
}

impl_wrap!(VECTOR_REDUCE_WRAP, VECTOR_REDUCE_NAME, vector_reduce, "vector-reduce");


fn id(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::PrarmsIsNotMatching(args));
    }
    Ok(args.get(0).unwrap().clone())
}

impl_wrap!(ID_WRAP, ID_NAME, id, "id");


fn ignore(_args: Vec<Value>) -> CResult {
    Ok(Value::Nil)
}

impl_wrap!(IGNORE_WRAP, IGNORE_NAME, ignore, "ignore");