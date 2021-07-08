mod ast;
mod error;
mod evaluation;
mod sexpr_to_ast;
mod value;
mod analysis;


use std::io::{stdin, stdout, Write};

use evaluation::Eval;
use sexpr_ir::syntax::sexpr::{file_parse, repl_parse};
use sexpr_to_ast::FromSexpr;

use crate::{ast::TopLevel, value::scope::Scope};


fn main() {
    let env = Scope::new();
    let prelude = file_parse("./scripts/boolean_algebra.sexpr").unwrap();
    let prelude: Result<Vec<_>, _> = prelude
        .iter()
        .map(TopLevel::from_sexpr)
        .collect();
    let prelude = prelude.unwrap();
    let prelude: Result<Vec<_>, _> = prelude.iter().map(|x| x.eval(&env)).collect();
    println!("load env: {:?}", prelude);
    loop {
        print!(">>> ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        // parse
        let r = repl_parse(&buf).unwrap();
        // into ast
        let r: Result<_, _> = TopLevel::from_sexpr(&r);
        match r {
            Err(e) => println!("error: {:?}", e),
            Ok(v) => {
                let r = v.eval(&env);
                match r {
                    Err(e) => println!("error: {:?}", e),
                    Ok(v) => println!("=>  {:?}", v),
                }
            }
        }
    }
}
