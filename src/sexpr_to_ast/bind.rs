use sexpr_ir::gast::{GAst, Handle, constant::Constant, list::List, symbol::Symbol};

use crate::{ast::{Expr, Let, Set, TopLevel}, error::{CompilerError, incomplete_expr, invalid_expr_length, invalid_expr_type, invalid_list_tail}, sexpr_to_ast::symbol_from_sexpr};

use super::FromSexpr;

impl FromSexpr<List, Set> for Set {
    fn from_sexpr(i: &List) -> Result<Set, Vec<CompilerError>> {
        let mut error_buffer = vec![];
        let mut iter = i.0.iter();

        let label = iter.next();

        if label.is_none() {
            error_buffer.push(incomplete_expr(&*i));
            return Err(error_buffer);
        }
        let pos = if let GAst::Const(Constant::Sym(l)) = label.unwrap() {
            l.1.clone()
        } else {
            error_buffer.push(invalid_expr_type(&*i, ()));
            return Err(error_buffer);
        };

        let name = iter.next();
        if name.is_none() {
            error_buffer.push(incomplete_expr(&*i));
            return Err(error_buffer);
        }
        let value = iter.next();
        if name.is_none() {
            error_buffer.push(incomplete_expr(&*i));
            return Err(error_buffer);
        }
        let name = name.unwrap();
        let value = value.unwrap();
        let name = if let GAst::Const(Constant::Sym(l)) = name {
            l.clone()
        } else {
            error_buffer.push(invalid_expr_type(&*i, ()));
            return Err(error_buffer);
        };
        let value = Expr::from_sexpr(value)?;
        Ok(Set{ name, value, pos })
    }
}

impl FromSexpr<List, Let> for Let {
    fn from_sexpr(i: &List) -> Result<Let, Vec<CompilerError>> {
        let mut error_buffer = vec![];

        // check is not tail
        if i.1.is_some() {
            error_buffer.push(invalid_list_tail(&*i));
        }

        let mut iter = i.0.iter();

        let label = iter.next();

        if label.is_none() {
            error_buffer.push(incomplete_expr(&*i));
            return Err(error_buffer);
        }
        let pos = if let GAst::Const(Constant::Sym(l)) = label.unwrap() {
            l.1.clone()
        } else {
            error_buffer.push(invalid_expr_type(&*i, ()));
            return Err(error_buffer);
        };

        // process binds
        let binds = iter
            .next()
            .ok_or_else(|| vec![incomplete_expr(&*i)])?
            .get_list()
            .ok_or_else(|| vec![invalid_expr_type(&*i, ())])?;

        if binds.1.is_some() {
            error_buffer.push(invalid_list_tail(&*i));
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
            Ok(Let { binds, body: bodys, pos })
        } else {
            Err(error_buffer)
        }
    }
}

fn bind_from_sexpr(i: &GAst) -> Result<(Handle<Symbol>, Expr), Vec<CompilerError>> {
    if let GAst::List(i) = i {
        let mut error_buffer = vec![];
        if i.1.is_some() {
            error_buffer.push(invalid_list_tail(&*i));
        }
        if i.0.len() != 2 {
            error_buffer.push(invalid_expr_length(i, 2, i.0.len()));
        }
        let name = i.0.get(0).unwrap();
        let expr = i.0.get(1).unwrap();
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
        Err(vec![invalid_expr_type(&*i, ())])
    }
}
