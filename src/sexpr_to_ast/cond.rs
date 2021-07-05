use sexpr_ir::gast::{GAst, list::List};

use crate::{ast::{Cond, Expr}, error::{CompilerError, bad_syntax}};

use super::FromSexpr;



impl FromSexpr<List, Cond> for Cond {
    fn from_sexpr(i: &List) -> Result<Cond, Vec<CompilerError>> {
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
                    pairs.pop();
                }
            }
        };

        if error_buffer.is_empty() {
            Ok(Cond { pairs, other })
        } else {
            Err(error_buffer)
        }
    }
}


fn pair_from_sexpr(i: &GAst) -> Result<(Expr, Expr), Vec<CompilerError>> {
    if let GAst::List(i) = i {
        let mut error_buffer = vec![];
        if i.1.is_some() {
            error_buffer.push(bad_syntax(&*i));
        }
        if i.0.len() != 2 {
            error_buffer.push(bad_syntax(&*i));
        }
        let cond = i.0.get(0).unwrap();
        let expr = i.0.get(0).unwrap();
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
        Err(vec![bad_syntax(i)])
    }
}