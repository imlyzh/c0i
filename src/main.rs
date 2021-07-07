mod ast;
mod error;
mod evaluation;
mod sexpr_to_ast;
mod value;
mod analysis;
mod prelude;


use std::io::{stdin, stdout, Write};

use prelude::init;
use evaluation::Eval;
use sexpr_ir::syntax::sexpr::{file_parse, repl_parse};
use sexpr_to_ast::FromSexpr;

use crate::ast::TopLevel;


fn main() {
    let env = init();
    let prelude = file_parse("./scripts/boolean_algebra.sexpr").unwrap();
    let prelude: Result<Vec<_>, _> = prelude
        .iter()
        .map(TopLevel::from_sexpr)
        .collect();
    let prelude = prelude.unwrap();
    let r = prelude
        .iter()
        .try_for_each(|x| x.eval(&env).map(|_| ()));
    if let Err(e) = r {
        println!("error: {:?}", e);
    }
    loop {
        print!(">>> ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        // parse
        let r = repl_parse(&buf).unwrap();
        // into ast
        match TopLevel::from_sexpr(&r) {
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
