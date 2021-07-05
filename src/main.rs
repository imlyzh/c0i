mod ast;
mod error;
mod eval;
mod sexpr_to_ast;
mod value;

// mod type_infer;

use std::io::{Write, stdin, stdout};

use sexpr_ir::syntax::sexpr::parse;
use sexpr_to_ast::FromSexpr;

use crate::ast::TopLevel;


fn main() {
    loop {
        print!(">>> ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let r = parse(&buf).unwrap();
        let r: Result<Vec<_>, _> = r.iter().map(TopLevel::from_sexpr).collect();
        match r {
            Ok(v) => println!("=>  {:?}", v),
            Err(e) => println!("error: {:?}", e),
        }
    }
}
