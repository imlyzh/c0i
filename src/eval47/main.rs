use std::env;
use std::sync::Arc;
use sexpr_ir::syntax::mexpr::file_parse;
use sexpr_ir::syntax::sexpr::parse;
use c0ilib::ast::TopLevel;
use c0ilib::eval47::min_scope_analysis::AnalyseContext;
use c0ilib::sexpr_to_ast::FromSexpr;

const BUILTINS: &'static str = include_str!("./builtins.scm");

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    let mut top_levels = Vec::new();
    eprintln!("Transforming builtins");
    let builtins = parse(BUILTINS, Arc::new("builtins".to_string())).unwrap();
    for piece in builtins {
        top_levels.push(TopLevel::from_sexpr(&piece).unwrap());
    }
    for arg in args.iter() {
        eprintln!("Transforming source file {}", arg);
        let file = file_parse(arg.as_str()).unwrap();
        for piece in file {
            top_levels.push(TopLevel::from_sexpr(&piece).unwrap())
        }
    }

    eprintln!("Performing analyse");
    let context = AnalyseContext::new();
    let analyse_result = context.min_scope_analyse(&top_levels);
    let result = serde_json::to_string_pretty(&analyse_result.data_collection).unwrap();
    eprintln!("{}", result);
}
