use std::collections::HashMap;
use std::convert::TryInto;
use pr47::data::tyck::TyckInfoPool;
use pr47::vm::al31f::insc::Insc;
use sexpr_ir::gast::Handle;
use xjbutil::boxed_slice;
use xjbutil::slice_arena::SliceArena;
use xjbutil::value::Value;
use crate::ast::{Expr, Function, TopLevel};
use crate::eval47::commons::{CompiledFunction, CompiledProgram, FFIAsyncFunction, FFIFunction};
use crate::eval47::data_map::GValue;
use crate::eval47::min_scope_analysis::AnalyseResult;
use crate::eval47::util::bitcast_i64_usize;

pub struct CompileContext {
    tyck_info_pool: TyckInfoPool,
    slice_arena: SliceArena<8192, 8>,
    code: Vec<Insc>,
    const_pool: Vec<Value>,
    init_proc: usize,
    functions: HashMap<usize, CompiledFunction>,
    ffi_funcs: Box<[FFIFunction]>,
    async_ffi_funcs: Box<[FFIAsyncFunction]>,

    compiling_function_chain: Vec<CompilingFunction>,
    compiling_function_names: Vec<String>
}

impl CompileContext {
    pub fn new(
        ffi_funcs: &[FFIFunction],
        async_ffi_funcs: &[FFIAsyncFunction]
    ) -> CompileContext {
        let context = CompileContext {
            tyck_info_pool: TyckInfoPool::new(),
            slice_arena: SliceArena::new(),
            code: Vec::new(),
            const_pool: Vec::new(),
            init_proc: 0,
            functions: HashMap::new(),
            ffi_funcs: ffi_funcs.iter()
                .map(|x| *x)
                .collect::<Vec<_ >>()
                .into_boxed_slice(),
            async_ffi_funcs: async_ffi_funcs.iter()
                .map(|x| *x)
                .collect::<Vec<_>>()
                .into_boxed_slice(),

            compiling_function_chain: Vec::new(),
            compiling_function_names: Vec::new()
        };
        context
    }

    pub fn compile(
        mut self,
        ast: &[TopLevel],
        analyse_result: &AnalyseResult
    ) -> CompiledProgram {
        for piece in ast {
            self.compile_top_level(piece, analyse_result);
        }

        todo!()
    }
}

struct CompilingFunction {
    start_addr: usize,
    capture_count: usize,
    arg_count: usize,
    local_count: usize
}

impl CompileContext {
    fn compile_top_level(&mut self, top_level: &TopLevel, analyse_result: &AnalyseResult) {
        if let TopLevel::Function(func_handle) = top_level {
            let func_id: i64 = analyse_result.data_collection
                .get(func_handle.as_ref(), "FunctionID")
                .unwrap()
                .clone()
                .try_into()
                .unwrap();
            let func_id = bitcast_i64_usize(func_id);
            let func_name: String = analyse_result.data_collection
                .get(func_handle.as_ref(), "FunctionName")
                .unwrap()
                .clone()
                .try_into()
                .unwrap();
            let param_var_ids: Vec<GValue> = analyse_result.data_collection
                .get(func_handle.as_ref(), "ParamVarIDs")
                .unwrap()
                .clone()
                .try_into()
                .unwrap();
            let base_frame_size: i64 = analyse_result.data_collection
                .get(func_handle.as_ref(), "BaseFrameSize")
                .unwrap()
                .clone()
                .try_into()
                .unwrap();
            let base_frame_size = bitcast_i64_usize(base_frame_size);

            self.compiling_function_names.push(func_name);
            self.display_compiling_function();

            let start_addr = self.code.len();
            let compiling_function = CompilingFunction {
                start_addr,
                capture_count: 0,
                arg_count: param_var_ids.len(),
                local_count: base_frame_size
            };
            self.compiling_function_chain.push(compiling_function);

            for stmt in func_handle.body.iter() {
                self.compile_stmt(stmt, analyse_result);
            }

            let compiling_function = self.compiling_function_chain.pop().unwrap();
            self.functions.insert(
                func_id,
                CompiledFunction {
                    start_addr: compiling_function.start_addr,
                    arg_count: compiling_function.arg_count,
                    ret_count: 1,
                    stack_size: compiling_function.local_count,

                    param_tyck_info: boxed_slice![],
                    exc_handlers: None
                }
            );
        } else {
            unreachable!()
        }
    }

    fn compile_function(&mut self, func: Handle<Function>, analyse_result: &AnalyseResult) {
        let func_id: i64 = analyse_result.data_collection
            .get(func.as_ref(), "FunctionID")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let func_id = bitcast_i64_usize(func_id);
        let func_name: String = analyse_result.data_collection
            .get(func.as_ref(), "FunctionName")
            .unwrap_or(&GValue::String("(anonymous)".into()))
            .clone()
            .try_into()
            .unwrap();
        let param_var_ids: Vec<GValue> = analyse_result.data_collection
            .get(func.as_ref(), "ParamVarIDs")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let base_frame_size: i64 = analyse_result.data_collection
            .get(func.as_ref(), "BaseFrameSize")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let base_frame_size = bitcast_i64_usize(base_frame_size);

        self.compiling_function_names.push(func_name);
        self.display_compiling_function();

        let start_addr = self.code.len();
        let compiling_function = CompilingFunction {
            start_addr,
            capture_count: 0,
            arg_count: param_var_ids.len(),
            local_count: base_frame_size
        };
        self.compiling_function_chain.push(compiling_function);

        for stmt in func.body.iter() {
            self.compile_stmt(stmt, analyse_result);
        }

    }

    fn compile_stmt(&mut self, stmt: &TopLevel, analyse_result: &AnalyseResult) {
        match stmt {
            TopLevel::Function(func_handle) => {
                self.compile_function(func_handle.clone(), analyse_result);
            },
            TopLevel::Bind(_, expr) => {
                let var_id: i64 = analyse_result.data_collection
                    .get(stmt, "VarID")
                    .unwrap()
                    .clone()
                    .try_into()
                    .unwrap();
                let var_id = bitcast_i64_usize(var_id);
                self.compile_expr(expr, analyse_result, false, Some(var_id));
            },
            TopLevel::Expr(expr) => self.compile_expr(expr, analyse_result, false, None)
        }
    }

    fn compile_expr(
        &mut self,
        _expr: &Expr,
        _analyse_result: &AnalyseResult,
        _called_as_function: bool,
        _tgt: Option<usize>
    ) {
    }

    fn display_compiling_function(&self) {
        for _ in 0..self.compiling_function_names.len() {
            eprint!("..");
        }
        eprint!(" ");
        for i in 0..self.compiling_function_names.len() {
            eprint!("{}", self.compiling_function_names[i]);
            if i != self.compiling_function_names.len() - 1 {
                eprint!("::");
            }
        }
        eprintln!();
    }
}
