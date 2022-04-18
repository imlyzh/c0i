use std::fs::File;
use std::io::{stdout, Write, stdin, Read};

use sexpr_ir::gast::Handle;

use crate::value::Value;
use crate::value::result::{CResult, CError};

pub(crate) fn read_stdin(args: Vec<Value>) -> CResult {
    if args.len() != 0 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let mut r = String::new();
    stdin().read_to_string(&mut r).unwrap();
    Ok(Value::Str(Handle::new(r)))
}

pub(crate) fn read_line(args: Vec<Value>) -> CResult {
    if args.len() != 0 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let mut r = String::new();
    stdin().read_line(&mut r).unwrap();
    Ok(Value::Str(Handle::new(r)))
}

pub(crate) fn display(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let r = args.get(0).unwrap().to_string();
    print!("{}", r);
    stdout().flush().unwrap();
    Ok(Value::Nil)
}

pub(crate) fn displayln(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let r = args.get(0).unwrap().to_string();
    println!("{}", r);
    Ok(Value::Nil)
}

pub(crate) fn file_to_string(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let v = if let Value::Str(v) = args.get(0).unwrap() {
        v
    } else {
        return CResult::Err(CError::TypeError((), args.get(0).unwrap().clone()));
    };
    let mut f = if let Ok(f) = File::open(v.as_str()) {
        f
    } else {
        return CResult::Err(CError::RuntimeError(Some(Value::Str(Handle::new(format!("file not found: {}", v))))));
    };
    let mut r = String::new();
    if let Err(e) = f.read_to_string(&mut r) {
        return CResult::Err(CError::RuntimeError(Some(Value::Str(Handle::new(format!("file read error: {}", e))))));
    }
    Ok(Value::Str(Handle::new(r)))
}

/*
pub(crate) fn file_to_lines(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let v = if let Value::Str(v) = args.get(0).unwrap() {
        v
    } else {
        return CResult::Err(CError::TypeError((), args.get(0).unwrap().clone()));
    };
    let mut f = if let Ok(f) = File::open(v.as_str()) {
        f
    } else {
        return CResult::Err(CError::RuntimeError(Some(Value::Str(Handle::new(format!("file not found: {}", v))))));
    };
    let mut r = String::new();
    if let Err(e) = f.read_to_string(&mut r) {
        return CResult::Err(CError::RuntimeError(Some(Value::Str(Handle::new(format!("file read error: {}", e))))));
    }
    Ok(Value::Str(Handle::new(r)))
}
 */

pub(crate) fn write_file(args: Vec<Value>) -> CResult {
    if args.len() != 2 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let v = if let Value::Str(v) = args.get(0).unwrap() {
        v
    } else {
        return CResult::Err(CError::TypeError((), args.get(0).unwrap().clone()));
    };
    let s = if let Value::Str(v) = args.get(0).unwrap() {
        v
    } else {
        return CResult::Err(CError::TypeError((), args.get(0).unwrap().clone()));
    };
    let mut f = if let Ok(f) = File::create(v.as_str()) {
        f
    } else {
        return CResult::Err(CError::RuntimeError(Some(Value::Str(Handle::new(format!("file not found: {}", v))))));
    };
    if let Err(e) = f.write_all(s.as_bytes()) {
        return CResult::Err(CError::RuntimeError(Some(Value::Str(Handle::new(format!("file write error: {}", e))))));
    }
    Ok(Value::Nil)
}
