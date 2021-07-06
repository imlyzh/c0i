use std::io::{Write, stdin, stdout};

use c0ilib::{analysis::free_variable::FreeVariables, ast::TopLevel, sexpr_to_ast::FromSexpr};
use sexpr_ir::{gast::Handle, syntax::sexpr::repl_parse};

fn main() {
    let mut env = vec![];
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
                let record: Vec<Handle<String>> = v.free_variables(&mut env)
                .iter()
                .map(|x|x.0.clone())
                .collect();
                println!("free variables: {:?}", record);
            }
        }
    }
}
