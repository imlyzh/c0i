use sexpr_ir::gast::{GAst, list::List};

use crate::{ast::{Call, Expr}, error::{CompilerError, bad_syntax}};

use super::FromSexpr;




impl FromSexpr<GAst, Call> for Call {
    fn from_sexpr(i: &GAst) -> Result<Call, Vec<CompilerError>> {
        if let GAst::List(x) = i {
            call_process(x)
        } else {
            Err(vec![bad_syntax(i)])
        }
    }
}


pub(crate) fn call_process(x: &List) -> Result<Call, Vec<CompilerError>> {
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
