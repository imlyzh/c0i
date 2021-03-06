use sexpr_ir::gast::{GAst, Handle, constant::Constant, list::List};

use crate::{ast::Expr, error::CompilerError, sexpr_to_ast::FromSexpr, value::{Pair, Value}};




pub fn quote_from_sexpr(i: &List) -> Result<Value, Vec<CompilerError>> {
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

pub fn value_from_sexpr(i: &GAst) -> Value {
    match i {
        GAst::Const(x) => {
            let r = Value::from_sexpr(x).unwrap();
            match r {
                Expr::Value(x) => x,
                Expr::Variable(n) => Value::Sym(n),
                _ => unreachable!(),
            }
        }
        GAst::List(i) => list_from_sexpr(i),
    }
}

pub fn list_from_sexpr(i: &List) -> Value {
    let List(i, pair_right) = i;
    let right = pair_right
        .clone()
        .map_or_else(|| Value::Nil, |x| value_from_sexpr(&x));
    let mut iter = i.iter().rev();
    let left = iter.next();
    if let Some(x) = left {
        let r = iter.fold(Pair(value_from_sexpr(&x), right), |prev, i| {
            Pair(value_from_sexpr(i), Value::Pair(Handle::new(prev)))
        });
        Value::Pair(Handle::new(r))
    } else {
        right
    }
}
