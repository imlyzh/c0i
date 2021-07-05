mod function;
mod top_level;

use sexpr_ir::gast::{constant::Constant, list::List, symbol::Symbol, GAst, Handle};

use crate::{
    ast::{Call, Expr},
    error::{bad_syntax, CompilerError},
    value::{Pair, Value},
};

use self::function::call_process;

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
        GAst::Const(Constant::Sym(n)) if *n.0 == "lambda" => {
            todo!()
        }
        GAst::Const(Constant::Sym(n)) if *n.0 == "let" => {
            todo!()
        }
        GAst::Const(Constant::Sym(n)) if *n.0 == "cond" => {
            todo!()
        }
        GAst::Const(Constant::Sym(n)) if *n.0 == "quote" => quote_from_sexpr(i).map(Expr::Value),
        _ => call_process(i).map(|x| Expr::FunctionCall(Handle::new(x))),
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

pub(crate) fn quote_from_sexpr(i: &List) -> Result<Value, Vec<CompilerError>> {
    let mut error_buffer = vec![];
    if i.1.is_some() {
        error_buffer.push(CompilerError::BadSyntax(i.to_string()));
    }
    if i.0.len() != 2 {
        error_buffer.push(CompilerError::BadSyntax(i.to_string()));
        return Err(error_buffer);
    }

    if let GAst::Const(Constant::Sym(x)) = i.0.get(0).unwrap() {
        if *x.0 == "quote" {
            error_buffer.push(CompilerError::BadSyntax(x.to_string()));
        }
    }
    let r = value_from_sexpr(i.0.get(1).unwrap());
    Ok(r)
}

fn value_from_sexpr(i: &GAst) -> Value {
    match i {
        GAst::Const(x) => {
            let r = Value::from_sexpr(x).unwrap();
            match r {
                Expr::Value(x) => x,
                Expr::Variable(n) => Value::Sym(n),
                _ => unreachable!(),
            }
        }
        GAst::List(i) => Value::Pair(Handle::new(list_from_sexpr(i))),
    }
}

fn list_from_sexpr(i: &List) -> Pair {
    let List(i, pair_right) = i;
    let right = pair_right
        .clone()
        .map_or_else(|| Value::Nil, |x| value_from_sexpr(&x));
    let mut iter = i.iter().rev();
    let left = iter
        .next()
        .map_or_else(|| Value::Nil, |x| value_from_sexpr(&x));
    iter.fold(Pair(left, right), |prev, i| {
        Pair(value_from_sexpr(i), Value::Pair(Handle::new(prev)))
    })
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
