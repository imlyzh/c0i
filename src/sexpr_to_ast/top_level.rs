use sexpr_ir::gast::{GAst, Handle, constant::Constant, list::List};

use crate::{ast::{Expr, Function, TopLevel}, error::CompilerError, sexpr_to_ast::{call_process, symbol_from_sexpr}, value::Value};

use super::FromSexpr;




impl FromSexpr<GAst, TopLevel> for TopLevel {
    fn from_sexpr(i: &GAst) -> Result<TopLevel, Vec<CompilerError>> {
        match i {
            GAst::Const(x) => Value::from_sexpr(x).map(TopLevel::Expr),
            GAst::List(x) => top_level_list_process(x),
        }
    }
}


pub(crate) fn top_level_list_process(x: &Handle<List>) -> Result<TopLevel, Vec<CompilerError>> {
    let mut error_buffer = vec![];
    if x.0.is_empty() {
        return Err(vec![CompilerError::BadSyntax(x.to_string())]);
    }
    match x.0.get(0).unwrap() {
        GAst::Const(Constant::Sym(n)) if *n.0 == "define" => {
            if x.1.is_some() {
                error_buffer.push(CompilerError::BadSyntax(x.to_string()));
            }
            if x.0.len() != 3 {
                error_buffer.push(CompilerError::BadSyntax(x.to_string()));
            }
            let name = x.0.get(1).unwrap();
            let expr = x.0.get(2).unwrap();
            let name = symbol_from_sexpr(name);
            let expr = Expr::from_sexpr(expr);
            if let Err(x) = name.clone() {
                error_buffer.push(x);
            }
            if let Err(mut x) = expr.clone() {
                error_buffer.append(&mut x);
            }
            if error_buffer.is_empty() {
                Ok(TopLevel::Bind(name.unwrap(), expr.unwrap()))
            } else {
                Err(error_buffer)
            }
        }
        GAst::Const(Constant::Sym(n)) if *n.0 == "defun" => {
            Function::from_sexpr(x).map(|x| TopLevel::Function(Handle::new(x)))
        }
        _ => call_process(x).map(|x| TopLevel::Expr(Expr::FunctionCall(Handle::new(x)))),
    }
}
