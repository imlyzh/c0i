pub mod quote;
mod function;
mod top_level;
mod call;
mod cond;
mod bind;

use sexpr_ir::gast::{constant::Constant, list::List, symbol::Symbol, GAst, Handle};

use crate::{ast::{Cond, Expr, Function, Let, Set}, error::CompilerError, value::Value};

use self::{call::call_process, quote::quote_from_sexpr};


pub trait FromSexpr<I, T> {
    fn from_sexpr(i: &I) -> Result<T, Vec<CompilerError>>;
}


impl FromSexpr<GAst, Expr> for Expr {
    fn from_sexpr(i: &GAst) -> Result<Expr, Vec<CompilerError>> {
        match i {
            GAst::Const(x) => Value::from_sexpr(x),
            GAst::List(x) => expr_list_process(x),
        }
    }
}

fn expr_list_process(i: &List) -> Result<Expr, Vec<CompilerError>> {
    // let mut error_buffer = vec![];
    if i.0.is_empty() {
        return Err(vec![CompilerError::BadSyntax(i.to_string())]);
    }
    match i.0.get(0).unwrap() {
        GAst::Const(Constant::Sym(n)) if *n.0 == "let" =>
            Let::from_sexpr(i).map(|f| Expr::Let(Handle::new(f))),
        GAst::Const(Constant::Sym(n)) if *n.0 == "set!" =>
            Set::from_sexpr(i).map(|f| Expr::Set(Handle::new(f))),
        GAst::Const(Constant::Sym(n)) if *n.0 == "cond" =>
            Cond::from_sexpr(i).map(|f| Expr::Cond(Handle::new(f))),
        GAst::Const(Constant::Sym(n)) if *n.0 == "lambda" =>
            Function::from_sexpr(i).map(|f| Expr::Lambda(Handle::new(f))),
        GAst::Const(Constant::Sym(n)) if *n.0 == "quote" => quote_from_sexpr(i).map(Expr::Value),
        _ => call_process(i).map(|x| Expr::FunctionCall(Handle::new(x))),
    }
}


macro_rules! ImplCastItem {
    ($i:expr, $name:ident) => {
        if let Constant::$name(x) = $i {
            return Ok(Expr::Value(Value::$name(x.clone())));
        }
    };
}

impl FromSexpr<Constant, Expr> for Value {
    fn from_sexpr(i: &Constant) -> Result<Expr, Vec<CompilerError>> {
        if let Constant::Nil = i {
            return Ok(Expr::Value(Value::Nil));
        }
        if let Constant::Sym(x) = i {
            return Ok(Expr::Variable(x.clone()));
        }
        ImplCastItem!(i, Bool);
        ImplCastItem!(i, Char);
        ImplCastItem!(i, Int);
        ImplCastItem!(i, Uint);
        ImplCastItem!(i, Float);
        ImplCastItem!(i, Str);
        unreachable!()
    }
}

fn symbol_from_sexpr(i: &GAst) -> Result<Handle<Symbol>, CompilerError> {
    i.get_const()
        .ok_or_else(|| CompilerError::IsNotSymbol(i.to_string()))?
        .get_sym()
        .ok_or_else(|| CompilerError::IsNotSymbol(i.to_string()))
}