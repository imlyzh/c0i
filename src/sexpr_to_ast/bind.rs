use sexpr_ir::gast::{GAst, Handle, list::List, symbol::Symbol};

use crate::{ast::{Expr, Let, TopLevel}, error::{CompilerError, bad_syntax}, sexpr_to_ast::symbol_from_sexpr};

use super::FromSexpr;


impl FromSexpr<List, Let> for Let {
    fn from_sexpr(i: &List) -> Result<Let, Vec<CompilerError>> {
        let mut error_buffer = vec![];

        // check is not tail
        if i.1.is_some() {
            error_buffer.push(bad_syntax(&*i));
        }

        let mut iter = i.0.iter();

        if iter.next().is_none() {
            error_buffer.push(bad_syntax(&*i));
            return Err(error_buffer);
        }

        // process binds
        let binds = iter
            .next()
            .ok_or_else(|| vec![bad_syntax(&*i)])?
            .get_list()
            .ok_or_else(|| vec![bad_syntax(&*i)])?;

        if binds.1.is_some() {
            error_buffer.push(bad_syntax(&*i));
        }

        // binds
        let binds = binds.0
            .iter()
            .map(bind_from_sexpr)
            .fold(vec![], |mut prev, i| {
            match i {
                Ok(v) => prev.push(v),
                Err(mut e) => error_buffer.append(&mut e),
            }
            prev
        });

        // process bodys

        let bodys = iter
            .map(TopLevel::from_sexpr)
            .fold(vec![], |mut prev, i| {
                match i {
                    Ok(v) => prev.push(v),
                    Err(mut e) => error_buffer.append(&mut e),
                }
                prev
            });

        if error_buffer.is_empty() {
            Ok(Let { binds, bodys })
        } else {
            Err(error_buffer)
        }
    }
}

fn bind_from_sexpr(i: &GAst) -> Result<(Handle<Symbol>, Expr), Vec<CompilerError>> {
    if let GAst::List(i) = i {
        let mut error_buffer = vec![];
        if i.1.is_some() {
            error_buffer.push(bad_syntax(&*i));
        }
        if i.0.len() != 2 {
            error_buffer.push(bad_syntax(&*i));
        }
        let name = i.0.get(0).unwrap();
        let expr = i.0.get(0).unwrap();
        let name = symbol_from_sexpr(name);
        let expr = Expr::from_sexpr(expr);
        if let Err(e) = name.clone() {
            error_buffer.push(e);
        }
        if let Err(mut e) = expr.clone() {
            error_buffer.append(&mut e);
        }

        if error_buffer.is_empty() {
            Ok((name.unwrap(), expr.unwrap()))
        } else {
            Err(error_buffer)
        }
    } else {
        Err(vec![bad_syntax(i)])
    }
}