mod ast;
mod error;
mod evaluation;
mod sexpr_to_ast;
mod value;
mod prelude;

use std::collections::HashMap;
pub use c0i::value::autobind;

use std::io::{stdin, stdout, Write};
use std::process::exit;

use prelude::init;
use evaluation::{Eval, load_file};
use sexpr_ir::gast::Handle;
use sexpr_ir::syntax::sexpr::repl_parse;
use sexpr_to_ast::FromSexpr;

use ast::TopLevel;
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
use libloading::{Library, Symbol};

type NativeModuleLoadFn = fn(
    &mut HashMap<Handle<crate::value::Symbol>, crate::value::Value>
);

fn main() {
    let mut args = env::args();
    args.next();
    let args = args.collect::<Vec<_>>();

    let env = if args.iter().any(|x| x == "--no-builtins") {
        Scope::new()
    } else {
        init()
    };

    let mut loaded_libraries = Vec::new();
    for arg in args {
        if arg.starts_with("--") {
            continue;
        }

        if arg.ends_with(".so") || arg.ends_with(".dll") {
            unsafe {
                let lib = Library::new(arg).unwrap();
                let sym: Symbol<NativeModuleLoadFn> = lib.get(b"load_module").unwrap();
                (sym)(&mut env.this_level.0.write().unwrap());

                loaded_libraries.push(lib);
            }
        } else {
            let r = load_file(&arg, &env);
            if let Err(e) = r {
                println!("error loading file {}: {:?}", arg, e);
            }
        }
    }
    start_repl(&env);
}
