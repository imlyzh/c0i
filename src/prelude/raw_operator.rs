use std::sync::RwLock;

use sexpr_ir::{gast::Handle, syntax::sexpr::parse};

use crate::{evaluation::call::Call, sexpr_to_ast::quote::value_from_sexpr, value::{Pair, Value, Vector, result::{CError, CResult}}};


pub(crate) fn read(args: Vec<Value>) -> CResult {
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

pub(crate) fn car(args: Vec<Value>) -> CResult {
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

pub(crate) fn cdr(args: Vec<Value>) -> CResult {
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

pub(crate) fn cons(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(2, args.len()));
    }
    Ok(Value::Pair(Handle::new(Pair(
            args.get(0).unwrap().clone(),
            args.get(1).unwrap().clone()))))
}

pub(crate) fn vector(args: Vec<Value>) -> CResult {
    Ok(Value::Vec(Vector(Handle::new(RwLock::new(
        args)))))
}

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

#[allow(dead_code)]
pub(crate) fn vector_reduce(args: Vec<Value>) -> CResult {
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

pub(crate) fn set_vector(args: Vec<Value>) -> CResult {
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

pub(crate) fn id(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    Ok(args.get(0).unwrap().clone())
}

pub(crate) fn ignore(_args: Vec<Value>) -> CResult {
    Ok(Value::Nil)
}
