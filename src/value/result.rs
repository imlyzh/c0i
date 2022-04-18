use std::fmt::Display;

use sexpr_ir::gast::{Handle, symbol::{Location, Symbol}};

use crate::value::Value;

use super::callable::Callable;


pub type CResult = Result<Value, CError>;


#[derive(Debug, Clone)]
pub enum CError {
    StackBacktrace(Callable, Handle<CError>),
    Positional(Location, Handle<CError>),
    SymbolNotFound(Handle<Symbol>),
    ValueIsNotCallable(Value),
    CondIsNotBoolean(Value),
    CondIsNotMatching,
    // CaptureVariableError(Handle<Symbol>),
    ArgsNotMatching(usize, usize),
    TypeError((), Value),
    // MathError,
    ZeroDivisionError,
    RuntimeError(Option<Value>),
    Unreachable(Option<Value>),
}

impl Display for CError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CError::StackBacktrace(t, e) => {
                writeln!(f, "\tfrom {}", t)?;
                e.fmt(f)
            },
            CError::Positional(e, t) => {
                writeln!(f, "\tat \"{}:{}:{}\"", e.path, e.line, e.colum)?;
                t.fmt(f)
            },
            CError::SymbolNotFound(e) => writeln!(f, "SymbolNotFound: {}.", e),
            CError::ValueIsNotCallable(e) => writeln!(f, "ValueIsNotCallable: {}.", e),
            CError::CondIsNotBoolean(e) => writeln!(f, "CondIsNotBoolean: {}.", e),
            CError::CondIsNotMatching => writeln!(f, "CondIsNotMatching."),
            CError::ArgsNotMatching(a, b) =>
                writeln!(f, "ArgsMatchingError: this function takes {} arguments but {} argument was supplied.",
                    a, b),
            CError::TypeError(e, v) =>
                writeln!(f, "TypeError: {:?} is not {:?} type.", v, e),
            CError::ZeroDivisionError => writeln!(f, "ZeroDivisionError."),
            CError::RuntimeError(e) => if let Some(e) = e {
                writeln!(f, "RuntimeError: {}.", e)
            } else {
                writeln!(f, "RuntimeError.")
            },
            CError::Unreachable(e) => if let Some(e) = e {
                writeln!(f, "Unreachable: {}.", e)
            } else {
                writeln!(f, "Unreachable.")
            },
        }
    }
}
