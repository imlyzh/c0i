use sexpr_ir::gast::{Handle, list::List};

use crate::{ast::{Call, Expr, Function, TopLevel}, error::{CompilerError, bad_syntax}, sexpr_to_ast::symbol_from_sexpr};

use super::FromSexpr;



impl FromSexpr<List, Function> for Function {
    fn from_sexpr(i: &List) -> Result<Function, Vec<CompilerError>> {
        let mut error_buffer = vec![];

        let mut iter = i.0.iter();

        // get def headle
        let def_headle = iter.next().ok_or_else(|| vec![bad_syntax(&*i)])?;
        let def_headle = symbol_from_sexpr(def_headle)
            .map_err(|e| vec![e])?
            .0
            .clone();

        // check is not tail
        if i.1.is_some() {
            error_buffer.push(bad_syntax(&*i));
        }

        // process prarms
        let prarms = iter
            .next()
            .ok_or_else(|| vec![bad_syntax(&*i)])?
            .get_list()
            .ok_or_else(|| vec![bad_syntax(&*i)])?;

        let List(prarms, extend_prarms) = (*prarms).clone();
        let mut prarms = prarms.iter();

        let function_name = if *def_headle == "defun" {
            let name = prarms.next().ok_or_else(|| vec![bad_syntax(&*i)])?;
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
        // /*
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
            };
            Ok(r)
        }
    }
}


pub(crate) fn call_process(x: &Handle<List>) -> Result<Call, Vec<CompilerError>> {
    let mut error_buffer = vec![];
    if x.1.is_some() {
        error_buffer.push(CompilerError::BadSyntax(x.to_string()));
    }
    let r =
        x.0.iter()
            .map(Expr::from_sexpr)
            .fold(vec![], |mut record, x| {
                if let Ok(x) = x {
                    record.push(x);
                } else if let Err(mut e) = x {
                    error_buffer.append(&mut e);
                }
                record
            });
    if error_buffer.is_empty() {
        Ok(Call(r))
    } else {
        Err(error_buffer)
    }
}