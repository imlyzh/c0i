use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::value::Value;


pub type CResult = Result<Value, CError>;


#[derive(Debug, Clone)]
pub enum CError {
    SymbolNotFound(Handle<Symbol>),
    ValueIsNotCallable(Value),
    CondIsNotFound(Value),
    CondIsNotMatching,
    CaptureVariableError(Handle<Symbol>),
    PrarmsIsNotMatching(Vec<Value>),
    TypeError(((), Value)),
}
