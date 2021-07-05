use sexpr_ir::gast::{constant::Constant, list::List, GAst, Handle};

use crate::{
    ast::{Expr, Function, TopLevel},
    error::CompilerError,
    sexpr_to_ast::symbol_from_sexpr,
    value::Value,
};

use super::{FromSexpr, quote_from_sexpr};

impl FromSexpr<GAst, TopLevel> for TopLevel {
    fn from_sexpr(i: &GAst) -> Result<TopLevel, Vec<CompilerError>> {
        match i {
            GAst::Const(x) => Value::from_sexpr(x).map(TopLevel::Expr),
            GAst::List(x) => top_level_list_process(x),
        }
    }
}

fn top_level_list_process(i: &List) -> Result<TopLevel, Vec<CompilerError>> {
    if i.0.is_empty() {
        return Err(vec![CompilerError::BadSyntax(i.to_string())]);
    }
    match i.0.get(0).unwrap() {
        GAst::Const(Constant::Sym(n)) if *n.0 == "define" => define_from_sexpr(i),
        GAst::Const(Constant::Sym(n)) if *n.0 == "defun" =>
            Function::from_sexpr(i).map(|f| TopLevel::Function(Handle::new(f))),
        GAst::Const(Constant::Sym(n)) if *n.0 == "quote" =>
            quote_from_sexpr(i).map(|v| TopLevel::Expr(Expr::Value(v))),
        _ => Expr::from_sexpr(&GAst::List(Handle::new(i.clone()))).map(TopLevel::Expr),
    }
}

fn define_from_sexpr(x: &List) -> Result<TopLevel, Vec<CompilerError>> {
    let mut error_buffer = vec![];
    if x.1.is_some() {
        error_buffer.push(CompilerError::BadSyntax(x.to_string()));
    }
    if x.0.len() != 3 {
        error_buffer.push(CompilerError::BadSyntax(x.to_string()));
        return Err(error_buffer);
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
