use sexpr_ir::gast::{constant::Constant, list::List, GAst, Handle};

use crate::{ast::{Expr, Function, TopLevel, ModuleTop, Import}, error::{CompilerError, incomplete_expr, invalid_expr_type}, sexpr_to_ast::symbol_from_sexpr, value::Value};

use super::{FromSexpr, quote_from_sexpr};


impl FromSexpr<GAst, ModuleTop> for ModuleTop {
    fn from_sexpr(i: &GAst) -> Result<ModuleTop, Vec<CompilerError>> {
        match i {
            GAst::Const(x) => Value::from_sexpr(x).map(TopLevel::Expr).map(ModuleTop::TopLevel),
            GAst::List(x) => module_top_list_process(x),
        }
    }
}

fn module_top_list_process(i: &List) -> Result<ModuleTop, Vec<CompilerError>> {
    if i.0.is_empty() {
        return Err(vec![CompilerError::BadSyntax(i.to_string())]);
    }
    match i.0.get(0).unwrap() {
        GAst::Const(Constant::Sym(n)) if *n.0 == "import" =>
            Import::from_sexpr(i).map(ModuleTop::Import),
        _ => TopLevel::from_sexpr(&GAst::List(Handle::new(i.clone()))).map(ModuleTop::TopLevel),
    }
}

impl FromSexpr<List, Import> for Import {
    fn from_sexpr(i: &List) -> Result<Import, Vec<CompilerError>> {
        let mut error_buffer = vec![];
        let mut iter = i.0.iter();

        let _label = iter.next();

        let name = iter.next();
        if name.is_none() {
            error_buffer.push(incomplete_expr(&*i));
            return Err(error_buffer);
        }
        let name = name.unwrap();
        let name = if let GAst::Const(Constant::Sym(l)) = name {
            l.clone()
        } else {
            error_buffer.push(invalid_expr_type(&*i, ()));
            return Err(error_buffer);
        };
        Ok(Import(name))
    }
}



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
        /*
        GAst::Const(Constant::Sym(n)) if *n.0 == "define" => define_from_sexpr(i),
        GAst::Const(Constant::Sym(n)) if *n.0 == "defun" =>
            Function::from_sexpr(i).map(|f| TopLevel::Function(Handle::new(f))),
         */
        GAst::Const(Constant::Sym(n)) if *n.0 == "define" =>
        if let Ok(f) = Function::from_sexpr(i){
            Ok(TopLevel::Function(Handle::new(f)))
        } else {
            define_from_sexpr(i)
        },
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
