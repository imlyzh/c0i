use std::sync::RwLock;

use lazy_static::lazy_static;

use sexpr_ir::{gast::{Handle, symbol::Symbol}, syntax::sexpr::parse};

use crate::{evaluation::call::Call, impl_wrap, sexpr_to_ast::quote::value_from_sexpr, value::{Pair, Value, Vector, callable::NativeFunction, result::{CError, CResult}}};

use super::LOCATION;


fn read(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let value = args.get(0).unwrap();
    if let Value::Str(str) = value {
        let r = parse(str, Handle::new("<read>".to_string()))
        .map_err(|_| CError::RuntimeError(Some(Value::Str(Handle::new("read function parse error".to_string())))))?;
        let r = r.first().ok_or(CError::RuntimeError(Some(Value::Str(Handle::new("read function parse error".to_string())))))?;
        let r = value_from_sexpr(r);
        Ok(r)
    } else {
        Err(CError::TypeError((), value.clone()))
    }
}

impl_wrap!(READ_WRAP, READ_NAME, read, "read", &LOCATION);

fn car(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let value = args.get(0).unwrap();
    if let Value::Pair(pair) = value {
        Ok(pair.0.clone())
    } else {
        Err(CError::TypeError((), value.clone()))
    }
}

impl_wrap!(CAR_WRAP, CAR_NAME, car, "car", &LOCATION);


fn cdr(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let value = args.get(0).unwrap();
    if let Value::Pair(pair) = value {
        Ok(pair.1.clone())
    } else {
        Err(CError::TypeError((), value.clone()))
    }
}

impl_wrap!(CDR_WRAP, CDR_NAME, cdr, "cdr", &LOCATION);


fn cons(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    Ok(Value::Pair(Handle::new(Pair(
            args.get(0).unwrap().clone(),
            args.get(1).unwrap().clone()))))
}

impl_wrap!(CONS_WRAP, CONS_NAME, cons, "cons", &LOCATION);


fn vector(args: Vec<Value>) -> CResult {
    Ok(Value::Vec(Vector(Handle::new(RwLock::new(
        args)))))
}

impl_wrap!(VECTOR_WRAP, VECTOR_NAME, vector, "make-vector", &LOCATION);

/*
fn vector_map(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::PrarmsIsNotMatching(2, args.len()));
    }
    let vector = args.get(1).unwrap();
    let callable = args.get(0).unwrap();
    let vector = if let Value::Vec(vec) = vector {
        vec
    } else {
        return Err(CError::TypeError((), vector.clone()));
    };
    let callable = if let Value::Callable(callable) = callable {
        callable
    } else {
        return Err(CError::TypeError((), callable.clone()));
    };
    let r: Result<Vec<_>, _> = vector.0.read().unwrap()
    .iter()
    .map(|x| callable.call(&[x.read().unwrap().clone()]))
    .collect();
    Ok(Value::Vec(Handle::new(Vector(r?))))
}

impl_wrap!(VECTOR_MAP_WRAP, VECTOR_MAP_NAME, vector_map, "vector-map", &LOCATION);
 */

fn vector_reduce(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let vector = args.get(1).unwrap();
    let callable = args.get(0).unwrap();
    let vector = if let Value::Vec(vec) = vector {
        vec
    } else {
        return Err(CError::TypeError((), vector.clone()));
    };
    let callable = if let Value::Callable(callable) = callable {
        callable
    } else {
        return Err(CError::TypeError((), callable.clone()));
    };
    let iter = vector.0.read().unwrap();
    let mut iter = iter.iter();
    let init = iter.next().map_or(Value::Nil, |x| x.clone());
    iter.try_fold(init, |x, y| callable.call(&[x, y.clone()]))
}

impl_wrap!(VECTOR_REDUCE_WRAP, VECTOR_REDUCE_NAME, set_vector, "vec-reduce", &LOCATION);

fn set_vector(args: Vec<Value>) -> CResult {
    if args.len() != 3 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    let vec = args.get(0).unwrap();
    let vec = if let Value::Vec(i) = vec {
        i
    } else {
        return Err(CError::TypeError((), vec.clone()));
    };
    let index = args.get(1).unwrap();
    let index = if let Value::Uint(i) = index {
        *i
    } else {
        return Err(CError::TypeError((), index.clone()));
    };
    let value = args.get(2).unwrap();
    vec.0.write().unwrap().insert(index as usize, value.clone());
    Ok(Value::Nil)
}

impl_wrap!(SET_VECTOR_WRAP, SET_VECTOR_NAME, set_vector, "set-vec!", &LOCATION);


fn id(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    Ok(args.get(0).unwrap().clone())
}

impl_wrap!(ID_WRAP, ID_NAME, id, "id", &LOCATION);


fn ignore(_args: Vec<Value>) -> CResult {
    Ok(Value::Nil)
}

impl_wrap!(IGNORE_WRAP, IGNORE_NAME, ignore, "ignore", &LOCATION);
