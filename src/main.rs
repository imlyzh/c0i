#![feature(unchecked_math)]
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
use sexpr_ir::gast::Handle;
use sexpr_ir::syntax::sexpr::{file_parse, repl_parse};
use sexpr_to_ast::FromSexpr;

use ast::TopLevel;
use value::result::CError;
use value::scope::Scope;


fn load_file(path: &str, env: &Handle<Scope>) -> Result<(), CError> {
    let file = file_parse(path).unwrap();
    let file: Result<Vec<_>, _> = file
        .iter()
        .map(TopLevel::from_sexpr)
        .collect();
    let file = file.unwrap();
    file
        .iter()
        .try_for_each(|x| x.eval(&env).map(|_| ()))?;
    Ok(())
}


fn main() {
    let env = init();
    let r = load_file("./scripts/boolean_algebra.sexpr", &env);
    if let Err(e) = r {
        println!("error: {:?}", e);
    }
    let r = load_file("./scripts/functools.sexpr", &env);
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
