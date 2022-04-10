use sexpr_ir::gast::{GAst, constant::Constant, list::List};

use crate::{ast::{Function, TopLevel}, error::{CompilerError, bad_syntax, incomplete_expr, invalid_expr_type, invalid_list_tail}, sexpr_to_ast::symbol_from_sexpr};

use super::FromSexpr;

impl FromSexpr<List, Function> for Function {
    fn from_sexpr(i: &List) -> Result<Function, Vec<CompilerError>> {
        let mut error_buffer = vec![];

        // check is not tail
        if i.1.is_some() {
            error_buffer.push(invalid_list_tail(&*i));
        }

        let mut iter = i.0.iter();

        // get def headle
        let def_headle = iter.next().ok_or_else(|| vec![incomplete_expr(&*i)])?;
        let (def_headle, pos) = if let GAst::Const(Constant::Sym(sym)) = def_headle {
            (sym.0.clone(), sym.1.clone())
        } else {
            error_buffer.push(invalid_expr_type(i, ()));
            return Err(error_buffer);
        };

        // process prarms
        let prarms = iter
            .next()
            .ok_or_else(|| vec![incomplete_expr(&*i)])?
            .get_list()
            .ok_or_else(|| vec![invalid_expr_type(i, ())])?;

        let List(prarms, extend_prarms) = (*prarms).clone();
        let mut prarms = prarms.iter();

        let function_name = if *def_headle == "define" {
            let name = prarms.next().ok_or_else(|| vec![incomplete_expr(&*i)])?;
            let name = symbol_from_sexpr(name);
            if let Err(e) = name.clone() {
                error_buffer.push(e);
            }
            Some(name)
        } else if *def_headle == "lambda" {
            None
        } else {
            error_buffer.push(bad_syntax(&*i));
            return Err(error_buffer);
        };

        // prarms
        let prarms = prarms.map(symbol_from_sexpr).fold(vec![], |mut pair, x| {
            if let Ok(x) = x {
                pair.push(x);
            } else if let Err(e) = x {
                error_buffer.push(e);
            }
            pair
        });

        // process extend prarms
        let extend_prarms = extend_prarms.map(|x| symbol_from_sexpr(&x));
        if let Some(Err(e)) = extend_prarms.clone() {
            error_buffer.push(e);
        }

        // process bodys
        let bodys: Vec<_> = iter.collect();
        /*
        if bodys.is_empty() {
            error_buffer.push(CompilerError::IncompleteExpr(i.to_string()));
            return Err(error_buffer);
        }
        // */
        let bodys = bodys
            .iter()
            .cloned()
            .map(TopLevel::from_sexpr)
            .fold(vec![], |mut pair, x| {
                if let Ok(x) = x {
                    pair.push(x);
                } else if let Err(mut e) = x {
                    error_buffer.append(&mut e);
                }
                pair
            });

        if !error_buffer.is_empty() {
            Err(error_buffer)
        } else {
            let r = Function {
                name: function_name.map(|x| x.unwrap()),
                prarms,
                extend_prarms: extend_prarms.map(|x| x.unwrap()),
                bodys,
                pos
            };
            Ok(r)
        }
    }
}
