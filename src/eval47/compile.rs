use pr47::vm::al31f::insc::Insc;
use xjbutil::slice_arena::SliceArena;
use xjbutil::value::Value;
use crate::ast::TopLevel;
use crate::eval47::commons::{CompiledFunction, CompiledProgram, FFIAsyncFunction, FFIFunction};
use crate::eval47::min_scope_analysis::AnalyseResult;

pub struct CompileContext {
    slice_arena: SliceArena<8192, 8>,
    code: Vec<Insc>,
    const_pool: Vec<Value>,
    init_proc: usize,
    functions: Vec<CompiledFunction>,
    ffi_funcs: Box<[FFIFunction]>,
    async_ffi_funcs: Box<[FFIAsyncFunction]>
}

impl CompileContext {
    pub fn new(
        ffi_funcs: &[FFIFunction],
        async_ffi_funcs: &[FFIAsyncFunction]
    ) -> CompileContext {
        let context = CompileContext {
            slice_arena: SliceArena::new(),
            code: vec![],
            const_pool: vec![],
            init_proc: 0,
            functions: vec![],
            ffi_funcs: ffi_funcs.iter()
                .map(|x| *x)
                .collect::<Vec<_ >>()
                .into_boxed_slice(),
            async_ffi_funcs: async_ffi_funcs.iter()
                .map(|x| *x)
                .collect::<Vec<_>>()
                .into_boxed_slice()
        };
        context
    }

    pub fn compile(
        mut self,
        _ast: &[TopLevel],
        _analyse_result: &AnalyseResult
    ) -> CompiledProgram {

        todo!()
    }
}

impl CompileContext {
    fn compile_top_level(&mut self, top_level: &TopLevel) {
        if let TopLevel::Function(func_handle) = top_level {

        } else {
            unreachable!()
        }
    }
}
