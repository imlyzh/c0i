#![feature(unchecked_math)]
mod ast;
mod error;
mod evaluation;
mod sexpr_to_ast;
mod value;
mod prelude;
// mod analysis;


use std::io::{stdin, stdout, Write};
use std::process::exit;

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
    if let Err(x) = file.clone() {
        println!("Complie Error: {:?}", x);
        exit(-1);
    }
    let file = file.unwrap();
    file
        .iter()
        .try_for_each(|x| x.eval(&env).map(|_| ()))?;
    Ok(())
}


fn start_repl(env: &Handle<Scope>) -> ! {
    loop {
        print!(">>> ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        if buf.trim().is_empty() {
            continue;
        }
        // parse
        let r = repl_parse(&buf).unwrap();
        // into ast
        match TopLevel::from_sexpr(&r) {
            Err(e) => {
                println!("error: {:?}", e);
                exit(-1)
            },
            Ok(v) => {
                let r = v.eval(&env);
                match r {
                    Err(e) => println!("Error:\n{}", e),
                    Ok(v) => println!("{}", v),
                }
            }
        }
    }
}


fn main() {
    let env = init();
    let std_list = [
        "./scripts/functools.sexpr"
    ];
    std_list.iter().for_each(|i| {
        let r = load_file(i, &env);
        if let Err(e) = r {
            println!("error: {:?}", e);
        }
    });
    start_repl(&env);
}
