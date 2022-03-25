use sexpr_ir::gast::{Handle, symbol::{Location, Symbol}};

use crate::value::Value;

#[derive(Debug, Clone)]
pub enum ModuleTop {
    Import(Import),
    TopLevel(TopLevel),
}

#[derive(Debug, Clone)]
pub struct Import(pub Handle<Symbol>);

#[derive(Debug, Clone)]
pub enum TopLevel {
    Function(Handle<Function>),
    Bind(Handle<Symbol>, Expr),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    Variable(Handle<Symbol>),
    Lambda(Handle<Function>),
    Let(Handle<Let>),
    Set(Handle<Set>),
    Cond(Handle<Cond>),
    FunctionCall(Handle<Call>),
}

#[derive(Debug, Clone)]
pub struct Let {
    pub binds: Vec<(Handle<Symbol>, Expr)>,
    pub bodys: Vec<TopLevel>,
    pub pos: Location,
}

#[derive(Debug, Clone)]
pub struct Set {
    pub name: Handle<Symbol>,
    pub value: Expr,
    pub pos: Location,
}

#[derive(Debug, Clone)]
pub struct Cond {
    pub pairs: Vec<(Expr, Expr)>,
    pub other: Option<Expr>,
    pub pos: Location,
}

#[derive(Debug, Clone)]
pub struct Call(pub Vec<Expr>);

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Option<Handle<Symbol>>,
    pub params: Vec<Handle<Symbol>>,
    pub extend_params: Option<Handle<Symbol>>,
    pub body: Vec<TopLevel>,
    pub pos: Location,
}
