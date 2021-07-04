mod function;
mod top_level;

use sexpr_ir::gast::{constant::Constant, list::List, symbol::Symbol, GAst, Handle};

use crate::{
    ast::{Call, Expr},
    error::{bad_syntax, CompilerError},
    value::Value,
};

use self::function::call_process;


pub trait FromSexpr<I, T> {
    fn from_sexpr(i: &I) -> Result<T, Vec<CompilerError>>;
}


impl FromSexpr<GAst, Expr> for Expr {
    fn from_sexpr(i: &GAst) -> Result<Expr, Vec<CompilerError>> {
        match i {
            GAst::Const(x) => Value::from_sexpr(x),
            GAst::List(x) => expr_list_process(x)
        }
    }
}


fn expr_list_process(x: &Handle<List>) -> Result<Expr, Vec<CompilerError>> {
    let mut error_buffer = vec![];
    if x.0.is_empty() {
        return Err(vec![CompilerError::BadSyntax(x.to_string())]);
    }
    match x.0.get(0).unwrap() {
        GAst::Const(Constant::Sym(n)) if *n.0 == "define" => {
            if x.1.is_some() {
                error_buffer.push(CompilerError::BadSyntax(x.to_string()));
            }
            todo!()
        }
        _ => call_process(x).map(|x| Expr::FunctionCall(Handle::new(x))),
    }
}


impl FromSexpr<GAst, Call> for Call {
    fn from_sexpr(i: &GAst) -> Result<Call, Vec<CompilerError>> {
        if let GAst::List(x) = i {
            call_process(x)
        } else {
            Err(vec![bad_syntax(i)])
        }
    }
}


fn symbol_from_sexpr(i: &GAst) -> Result<Handle<Symbol>, CompilerError> {
    i.get_const()
        .ok_or_else(|| CompilerError::IsNotSymbol(i.to_string()))?
        .get_sym()
        .ok_or_else(|| CompilerError::IsNotSymbol(i.to_string()))
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
