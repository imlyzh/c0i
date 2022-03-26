use crate::ast::TopLevel;
use crate::eval47::commons::CompiledProgram;
use crate::eval47::min_scope_analysis::AnalyseResult;

pub struct CompileContext {

}

impl CompileContext {
    pub fn new() -> CompileContext {
        CompileContext {}
    }

    pub fn compile(&self, _ast: &[TopLevel], _analyse_result: AnalyseResult) -> CompiledProgram {
        todo!()
    }
}
