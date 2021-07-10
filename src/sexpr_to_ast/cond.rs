use sexpr_ir::gast::{GAst, constant::Constant, list::List};

use crate::{ast::{Cond, Expr}, error::{CompilerError, incomplete_expr, invalid_expr_length, invalid_expr_type, invalid_list_tail}};

use super::FromSexpr;



impl FromSexpr<List, Cond> for Cond {
    fn from_sexpr(i: &List) -> Result<Cond, Vec<CompilerError>> {
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

        let mut pairs = iter
            .map(pair_from_sexpr)
            .fold(vec![], |mut prev, i| {
            match i {
                Ok(v) => prev.push(v),
                Err(mut e) => error_buffer.append(&mut e),
            }
            prev
        });

        let mut other = None;
        if !pairs.is_empty() {
            if let (Expr::Variable(s), o) = pairs.last().unwrap() {
                if *s.0 == "else" {
                    other = Some(o.clone());
                    pairs= pairs[..pairs.len()-1].to_vec();
                }
            }
        };

        if error_buffer.is_empty() {
            Ok(Cond { pairs, other, pos })
        } else {
            Err(error_buffer)
        }
    }
}


fn pair_from_sexpr(i: &GAst) -> Result<(Expr, Expr), Vec<CompilerError>> {
    if let GAst::List(i) = i {
        let mut error_buffer = vec![];
        if i.1.is_some() {
            error_buffer.push(invalid_list_tail(&*i));
        }
        if i.0.len() != 2 {
            error_buffer.push(invalid_expr_length(i, 2, i.0.len()));
        }
        let cond = i.0.get(0).unwrap();
        let expr = i.0.get(1).unwrap();
        let cond = Expr::from_sexpr(cond);
        let expr = Expr::from_sexpr(expr);
        if let Err(mut e) = cond.clone() {
            error_buffer.append(&mut e);
        }
        if let Err(mut e) = expr.clone() {
            error_buffer.append(&mut e);
        }

        if error_buffer.is_empty() {
            Ok((cond.unwrap(), expr.unwrap()))
        } else {
            Err(error_buffer)
        }
    } else {
        Err(vec![invalid_expr_type(i, ())])
    }
}