
use std::fs::File;
use std::io::{stdout, Write, stdin, Read};

use lazy_static::lazy_static;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::impl_wrap;
use crate::value::Value;
use crate::value::callable::NativeFunction;
use crate::value::result::{CResult, CError};

use super::LOCATION;



fn read_stdin(args: Vec<Value>) -> CResult {
    if args.len() != 0 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let mut r = String::new();
    stdin().read_to_string(&mut r).unwrap();
    Ok(Value::Str(Handle::new(r)))
}

fn read_line(args: Vec<Value>) -> CResult {
    if args.len() != 0 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let mut r = String::new();
    stdin().read_line(&mut r).unwrap();
    Ok(Value::Str(Handle::new(r)))
}

impl_wrap!(READ_STDIN_WRAP, READ_STDIN_NAME, read_stdin, "read-stdin", &LOCATION);
impl_wrap!(READ_LINE_WRAP, READ_LINE_NAME, read_line, "read-line", &LOCATION);


fn display(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let r = args.get(0).unwrap().to_string();
    print!("{}", r);
    stdout().flush().unwrap();
    Ok(Value::Nil)
}

fn displayln(args: Vec<Value>) -> CResult {
    if args.len() != 1 {
        return Err(CError::ArgsNotMatching(1, args.len()));
    }
    let r = args.get(0).unwrap().to_string();
    println!("{}", r);
    Ok(Value::Nil)
}

impl_wrap!(DISPLAY_WRAP, DISPLAY_NAME, display, "display", &LOCATION);
impl_wrap!(DISPLAYLN_WRAP, DISPLAYLN_NAME, displayln, "displayln", &LOCATION);


fn file_to_string(args: Vec<Value>) -> CResult {
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
fn file_to_lines(args: Vec<Value>) -> CResult {
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

impl_wrap!(FILE_TO_STRING_WRAP, FILE_TO_STRING_NAME, file_to_string, "file->string", &LOCATION);
// impl_wrap!(FILE_TO_LINES_WRAP, FILE_TO_LINES_NAME, file_to_lines, "file-lines", &LOCATION);


fn write_file(args: Vec<Value>) -> CResult {
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

impl_wrap!(WRITE_FILE_WRAP, WRITE_FILE_NAME, write_file, "write-file", &LOCATION);