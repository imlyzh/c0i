use sexpr_ir::gast::{symbol::Symbol, Handle};

use super::result::CResult;
use super::scope::Scope;
use crate::ast::Function;

use super::Value;

#[derive(Debug, Clone)]
pub struct Closure(Function, Option<Handle<Scope>>);

type NativeInterface = fn(Vec<Value>) -> CResult;

#[derive(Debug, Clone)]
pub struct NativeFunction {
    name: Handle<Symbol>,
    is_pure: bool,
    type_sign: (),
    interface: NativeInterface,
}

#[derive(Debug, Clone)]
pub enum Callable {
    Function(Handle<Closure>),
    Native(Handle<NativeFunction>),
}
pub trait Call {
    fn call(&self, args: Vec<Value>) -> CResult;
}

impl Call for Callable {
    fn call(&self, args: Vec<Value>) -> CResult {
        todo!()
    }
}

impl Call for Closure {
    fn call(&self, args: Vec<Value>) -> CResult {
        todo!()
    }
}

impl Call for NativeFunction {
    fn call(&self, i: Vec<Value>) -> CResult {
        (self.interface)(i)
    }
}
