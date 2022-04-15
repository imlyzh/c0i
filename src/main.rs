mod ast;
mod error;
mod evaluation;
mod sexpr_to_ast;
mod value;
mod prelude;

pub use prelude::autobind;

use std::io::{stdin, stdout, Write};
use std::process::exit;

use prelude::init;
use evaluation::{Eval, load_file};
use sexpr_ir::gast::Handle;
use sexpr_ir::syntax::sexpr::{
    // file_parse,
    repl_parse
};
use sexpr_to_ast::FromSexpr;

use ast::TopLevel;
// use value::result::CError;
use value::scope::Scope;

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
        let r = repl_parse(&buf);
        if r.is_err() {
            println!("syntax error: {}", r.unwrap_err());
            continue;
        }
        let r = r.unwrap();
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

use std::env;

fn main() {
    let env = init();
    /*
    let std_list = [
        "./scripts/functools.sexpr"
    ];
     */
    let mut args = env::args();
    args.next();

    args.for_each(|i| {
        let r = load_file(&i, &env);
        if let Err(e) = r {
            println!("error: {:?}", e);
        }
    });
    start_repl(&env);
}
