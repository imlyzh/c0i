use std::convert::TryInto;
use std::env;
use std::sync::Arc;
use pr47::data::exception::Exception;
use pr47::vm::al31f::alloc::default_alloc::DefaultAlloc;
use pr47::vm::al31f::executor::{create_vm_main_thread, vm_thread_run_function, VMThread};
use sexpr_ir::syntax::mexpr::file_parse;
use sexpr_ir::syntax::sexpr::parse;
use xjbutil::std_ext::ResultExt;
use xjbutil::unchecked::UncheckedSendSync;
use c0ilib::ast::TopLevel;
use c0ilib::eval47::builtins::DBG_INT_BIND;
use c0ilib::eval47::commons::CompiledProgram;
use c0ilib::eval47::compile::CompileContext;
use c0ilib::eval47::min_scope_analysis::AnalyseContext;
use c0ilib::eval47::util::bitcast_i64_usize;
use c0ilib::sexpr_to_ast::FromSexpr;

const BUILTINS: &'static str = include_str!("./builtins.scm");

async fn run_program(
    program: CompiledProgram,
    func_id: usize,
    args: Vec<pr47::data::Value>
) {
    for _ in 0..10 {
        let alloc: DefaultAlloc = DefaultAlloc::new();
        let mut vm_thread: Box<VMThread<DefaultAlloc>> =
            create_vm_main_thread(alloc, &program).await;

        let start_time = std::time::Instant::now();
        let result: Result<Vec<pr47::data::Value>, Exception> = unsafe {
            vm_thread_run_function::<DefaultAlloc, false>(
                UncheckedSendSync::new((&mut vm_thread, func_id, &args))
            ).unwrap_no_debug().await.into_inner()
        };
        let end_time = std::time::Instant::now();
        eprintln!("elapsed time = {}", (end_time - start_time).as_millis());

        result.unwrap_no_debug();
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    let mut top_levels = Vec::new();
    eprintln!("Transforming builtins");
    let builtins = parse(BUILTINS, Arc::new("builtins".to_string())).unwrap();
    for piece in builtins {
        top_levels.push(TopLevel::from_sexpr(&piece).unwrap());
    }
    for arg in args.iter() {
        if arg.starts_with("--") {
            continue;
        }

        eprintln!("Transforming source file `{}`", arg);
        let file = file_parse(arg.as_str()).expect("Failed parsing file to SExpr");
        for piece in file {
            top_levels.push(TopLevel::from_sexpr(&piece)
                .expect("Failed transforming SExpr to TopLevel AST"));
        }
    }

    eprintln!("Performing analyse");
    let mut context = AnalyseContext::new();
    context.register_ffi("dbg-int", &DBG_INT_BIND);
    let analyse_result = context.min_scope_analyse(&top_levels);

    if args.contains(&"--only-analyse".to_string()) {
        let data_collection =
            serde_json::to_string_pretty(&analyse_result.data_collection).unwrap();
        println!("\ndata_collection = {}", data_collection);

        let functions = serde_json::to_string_pretty(&analyse_result.functions).unwrap();
        println!("\nfunctions = {}", functions);
        return;
    }

    let mut ffi_functions_in_use = analyse_result.ffi_function_in_use.iter()
        .collect::<Vec<_>>();
    ffi_functions_in_use.sort_by_key(|x| x.0);
    let ffi_functions_in_use = ffi_functions_in_use.into_iter()
        .map(|(_, v)| v.0)
        .collect::<Vec<_>>();

    let mut async_ffi_functions_in_use = analyse_result.async_ffi_function_in_use.iter()
        .collect::<Vec<_>>();
    async_ffi_functions_in_use.sort_by_key(|x| x.0);
    let async_ffi_functions_in_use = async_ffi_functions_in_use.into_iter()
        .map(|(_, v)| v.0)
        .collect::<Vec<_>>();

    let compile_context = CompileContext::new(
        &ffi_functions_in_use,
        &async_ffi_functions_in_use,
    );
    let result = compile_context.compile(&top_levels, &analyse_result);

    let application_start =
        analyse_result.global_data_map.get("ApplicationStartFuncID")
            .expect("no `application-start` procedure found in the program")
            .clone()
            .try_into()
            .unwrap();
    let application_start = bitcast_i64_usize(application_start);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
        .block_on(run_program(
            result.cp,
            application_start,
            vec![],
        ));
}
