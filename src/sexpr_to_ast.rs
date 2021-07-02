

use sexpr_ir::gast::{GAst, Handle, constant::Constant, list::List, symbol::Symbol};

use crate::{ast::{Expr, Function, TopLevel}, error::{CompilerError, bad_syntax}, value::Value};


pub trait FromSexpr<I, T> {
    fn from_sexpr(i: &I) -> Result<T, Vec<CompilerError>>;
}


macro_rules! ImplCastItem {
    ($i:expr, $name:ident) => {
        if let Constant::$name(x) = $i {
            return Ok(Expr::Value(Value::$name(x.clone())));
        }
    };
}


impl FromSexpr<GAst, Expr> for Expr {
    fn from_sexpr(i: &GAst) -> Result<Expr, Vec<CompilerError>> {
        match i {
            GAst::Const(x) => Value::from_sexpr(x),
            GAst::List(x) => {
                if x.0.is_empty() {
                    return Err(vec![CompilerError::BadSyntax(x.to_string())]);
                }
                todo!()
            },
        }
    }
}

impl FromSexpr<GAst, TopLevel> for TopLevel {
    fn from_sexpr(i: &GAst) -> Result<TopLevel, Vec<CompilerError>> {
        match i {
            GAst::Const(x) => Value::from_sexpr(x).map(TopLevel::Expr),
            GAst::List(x) => {
                if x.0.is_empty() {
                    return Err(vec![CompilerError::BadSyntax(x.to_string())]);
                }
                todo!()
            },
        }
    }
}


impl FromSexpr<List, Function> for Function {
    fn from_sexpr(i: &List) -> Result<Function, Vec<CompilerError>> {
        let mut error_buffer = vec![];

        let mut iter = i.0.iter();

        // get def headle
        let def_headle = iter
            .next()
            .ok_or_else(||vec![bad_syntax(&*i)])?;
        let def_headle = symbol_from_sexpr(def_headle)
            .map_err(|e| vec![e])?
            .0.clone();
        
        // check is not tail
        if i.1.is_some() {
            error_buffer.push(bad_syntax(&*i));
        }

        // process prarms
        let prarms = iter
            .next()
            .ok_or_else(||vec![bad_syntax(&*i)])?
            .get_list()
            .ok_or_else(||vec![bad_syntax(&*i)])?;
        
        let List(prarms, extend_prarms) = (*prarms).clone();
        let mut prarms = prarms.iter();
        
        let function_name = if *def_headle == "defun" {
            let name = prarms.next().ok_or_else(||vec![bad_syntax(&*i)])?;
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
        let prarms = prarms
            .map(symbol_from_sexpr)
            .fold(vec![], |mut pair, x| {
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
                prarms: prarms,
                extend_prarms: extend_prarms.map(|x| x.unwrap()),
                bodys: bodys,
            };
            Ok(r)
        }
    }
}


fn symbol_from_sexpr(i: &GAst) -> Result<Handle<Symbol>, CompilerError> {
    i
    .get_const().ok_or_else(||CompilerError::IsNotSymbol(i.to_string()))?
    .get_sym().ok_or_else(||CompilerError::IsNotSymbol(i.to_string()))
}


impl FromSexpr<Constant, Expr> for Value { 
    fn from_sexpr(i: &Constant) -> Result<Expr, Vec<CompilerError>> {
        if let Constant::Nil = i {
            return Ok(Expr::Value(Value::Nil));
        }
        if let Constant::Sym(x) = i {
            return Ok(Expr::Variable(x.clone()));
        }
        ImplCastItem!(i, Bool);
        ImplCastItem!(i, Char);
        ImplCastItem!(i, Int);
        ImplCastItem!(i, Uint);
        ImplCastItem!(i, Float);
        ImplCastItem!(i, Str);
        unreachable!()
    }
}