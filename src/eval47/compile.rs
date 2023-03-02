use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;
use std::ptr::NonNull;
use pr47::builtins::closure::Closure;
use pr47::builtins::vec::{vec_ctor, VMGenericVec};
use pr47::data::generic::GenericTypeVT;
use pr47::data::tyck::{TyckInfo, TyckInfoPool};
use pr47::data::wrapper::{OWN_INFO_READ_MASK, OWN_INFO_WRITE_MASK, OwnershipInfo};
use pr47::vm::al31f::insc::Insc;
use sexpr_ir::gast::Handle;
use sexpr_ir::gast::symbol::Symbol;
use xjbutil::boxed_slice;
use xjbutil::korobka::Korobka;
use xjbutil::slice_arena::SliceArena;
use crate::ast::{Call, Cond, Expr, Function, Let, Set, TopLevel};
use crate::eval47::commons::{CompiledFunction, CompiledProgram, FFIAsyncFunction, FFIFunction};
use crate::eval47::data_map::GValue;
use crate::eval47::min_scope_analysis::AnalyseResult;
use crate::eval47::util::{bitcast_i64_usize, MantisGod};
use crate::{guard, guard2};
use crate::eval47::util::Guard;
use crate::value::Value;

pub struct CompileContext {
    tyck_info_pool: TyckInfoPool,
    vt_pool: Vec<Korobka<GenericTypeVT>>,
    slice_arena: SliceArena<8192, 8>,
    code: Vec<Insc>,
    const_pool: Vec<pr47::data::Value>,
    init_proc: usize,
    functions: HashMap<usize, CompiledFunction>,
    ffi_funcs: Box<[FFIFunction]>,
    async_ffi_funcs: Box<[FFIAsyncFunction]>,

    compiling_function_chain: Vec<CompilingFunction>,
    compiling_function_names: Vec<String>,
    compiling_loops: Vec<LoopContext>,

    func_queue: VecDeque<(Vec<String>, Handle<Function>)>,
}

pub struct CompileResult {
    pub cp: CompiledProgram,
    #[allow(dead_code)]
    tyck_info_pool: TyckInfoPool,
    #[allow(dead_code)]
    vt_pool: Vec<Korobka<GenericTypeVT>>
}

impl CompileResult {
    pub fn program(&self) -> &CompiledProgram {
        &self.cp
    }
}

impl CompileContext {
    pub fn new(
        ffi_funcs: &[FFIFunction],
        async_ffi_funcs: &[FFIAsyncFunction]
    ) -> CompileContext {
        let context = CompileContext {
            tyck_info_pool: TyckInfoPool::new(),
            vt_pool: Vec::new(),
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
            compiling_function_names: Vec::new(),
            compiling_loops: Vec::new(),

            func_queue: VecDeque::new()
        };
        context
    }

    pub fn compile(
        mut self,
        ast: &[TopLevel],
        analyse_result: &mut AnalyseResult
    ) -> CompileResult {
        eprintln!("Compiling to bytecode");
        for global_const in analyse_result.global_consts.iter() {
            let value = match global_const {
                GValue::Nil => pr47::data::Value::new_bool(false),
                GValue::Bool(b) => pr47::data::Value::new_bool(*b),
                GValue::Int(i) => pr47::data::Value::new_int(*i),
                GValue::Float(f) => pr47::data::Value::new_float(*f),
                GValue::String(s) => unsafe {
                    let v = pr47::data::Value::new_owned(s.clone());
                    v.set_ownership_info(OwnershipInfo::GlobalConst);
                    v
                },
                _ => unreachable!()
            };
            self.const_pool.push(value);
        }


        for piece in ast {
            if let TopLevel::Function(func) = piece {
                self.func_queue.push_back((self.compiling_function_names.clone(), func.clone()));
            }
        }
        while let Some((function_names, func_handle)) = self.func_queue.pop_front() {
            self.compiling_function_names = function_names;
            self.compile_function(func_handle, analyse_result);
        }

        let mut functions = self.functions.into_iter().collect::<Vec<_>>();
        functions.sort_by_key(|x| x.0);
        let functions = functions.into_iter().map(|x| x.1).collect::<Vec<_>>().into_boxed_slice();

        CompileResult {
            cp: CompiledProgram {
                slice_arena: self.slice_arena,
                code: self.code.into_boxed_slice(),
                const_pool: self.const_pool.into_boxed_slice(),
                init_proc: self.init_proc,
                functions,
                ffi_funcs: self.ffi_funcs,
                async_ffi_funcs: self.async_ffi_funcs
            },
            tyck_info_pool: self.tyck_info_pool,
            vt_pool: self.vt_pool
        }
    }
}

struct CompilingFunction {
    #[allow(dead_code)]
    start_addr: usize,
    capture_count: usize,
    #[allow(dead_code)]
    arg_count: usize,
    local_count: usize
}

impl CompilingFunction {
    fn allocate_temp(&mut self) -> usize {
        let ret = self.local_count;
        self.local_count += 1;
        ret
    }

    fn translate_local_id_to_address(&self, id: usize) -> usize {
        id + self.capture_count
    }
}

struct LoopContext {
    loop_start_addr: usize,
    pending_breaks: Vec<usize>
}

impl LoopContext {
    pub fn new(loop_start_addr: usize) -> Self {
        Self {
            loop_start_addr,
            pending_breaks: Vec::new()
        }
    }
}

impl CompileContext {
    fn format_function_chain(&self) -> String {
        let mut chain = String::new();
        for name in self.compiling_function_names.iter() {
            chain.push_str(name);
            chain.push_str("::");
        }
        chain
    }

    fn compile_function(&mut self, func: Handle<Function>, analyse_result: &mut AnalyseResult) {
        let func_id: i64 = analyse_result.data_collection
            .get(func.as_ref(), "FunctionID")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let func_id = bitcast_i64_usize(func_id);
        let func_name: String = analyse_result.data_collection
            .get(func.as_ref(), "FunctionName")
            .unwrap_or(&GValue::String("<anonymous>".into()))
            .clone()
            .try_into()
            .unwrap();

        let function_chain = self.format_function_chain();
        let mut g = guard2!(
            func.pos,
            "compile function `{}{}` (func_id = {})",
            function_chain.clone(),
            func_name,
            func_id
        );
        analyse_result.functions.insert_raw_key(
            func_id,
            "ResolvedFunctionName",
            function_chain + func_name.as_str()
        );
        analyse_result.functions.insert_raw_key(
            func_id,
            "ResolvedFunctionLocation",
            format!("file \"{}\", line {}, column {}", func.pos.path, func.pos.line, func.pos.colum)
        );

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
        let captures: Vec<GValue> = analyse_result.data_collection
            .get(func.as_ref(), "Captures")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let base_frame_size = bitcast_i64_usize(base_frame_size);

        self.compiling_function_names.push(func_name);

        let start_addr = self.code.len();
        let compiling_function = CompilingFunction {
            start_addr,
            capture_count: captures.len(),
            arg_count: param_var_ids.len(),
            local_count: base_frame_size
        };
        self.compiling_function_chain.push(compiling_function);

        let ret_pos = self.compile_stmt_list(&func.body, analyse_result);
        if let Some(ret_pos) = ret_pos {
            self.code.push(Insc::ReturnOne(ret_pos));
        } else {
            let ret_pos = self.compiling_function_chain.last_mut().unwrap()
                .allocate_temp();
            self.code.push(Insc::MakeBoolConst(false, ret_pos));
            self.code.push(Insc::ReturnOne(ret_pos));
        }

        let compiling_function = self.compiling_function_chain.pop().unwrap();
        self.compiling_function_names.pop().unwrap();
        self.functions.insert(func_id, CompiledFunction {
            start_addr,
            arg_count: param_var_ids.len(),
            ret_count: 1,
            stack_size: compiling_function.local_count,
            param_tyck_info: boxed_slice![],
            exc_handlers: None
        });

        g.cancel();
    }

    fn compile_stmt_list(
        &mut self,
        stmt_list: &[TopLevel],
        analyse_result: &mut AnalyseResult
    ) -> Option<usize> {
        let mut ret = None;
        for stmt in stmt_list {
            ret = self.compile_stmt(stmt, analyse_result, None);
        }
        ret
    }

    fn compile_stmt(
        &mut self,
        stmt: &TopLevel,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> Option<usize> {
        match stmt {
            TopLevel::Function(func_handle) => {
                self.func_queue.push_back((
                    self.compiling_function_names.clone(),
                    func_handle.clone()
                ));
                None
            },
            TopLevel::Bind(_, expr) => {
                let var_id: i64 = analyse_result.data_collection
                    .get(stmt, "VarID")
                    .unwrap()
                    .clone()
                    .try_into()
                    .unwrap();
                let var_id = bitcast_i64_usize(var_id);
                self.compile_expr(expr, analyse_result, Some(var_id));
                None
            },
            TopLevel::Expr(expr) => Some(self.compile_expr(expr, analyse_result, tgt))
        }
    }

    fn compile_expr(
        &mut self,
        expr: &Expr,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        match expr {
            Expr::Value(value) => self.compile_value(value, analyse_result, tgt),
            Expr::Variable(var) => self.compile_var(var.clone(), analyse_result, tgt),
            Expr::Lambda(lambda) => self.compile_lambda(lambda.clone(), analyse_result, tgt),
            Expr::Let(let_item) => self.compile_let(let_item.clone(), analyse_result, tgt),
            Expr::Set(set) => self.compile_set(set.clone(), analyse_result, tgt),
            Expr::Cond(cond) => self.compile_cond(cond.clone(), analyse_result, tgt),
            Expr::FunctionCall(call) => self.compile_call(call.clone(), analyse_result, tgt)
        }
    }

    fn compile_expr_for_fn_call(
        &mut self,
        expr: &Expr,
        analyse_result: &mut AnalyseResult
    ) -> MantisGod<usize, usize, (bool, usize)> {
        match expr {
            Expr::Variable(var) => self.compile_var_for_fn_call(var.clone(), analyse_result),
            _ => MantisGod::Left(self.compile_expr(expr, analyse_result, None))
        }
    }

    fn compile_value(
        &mut self,
        value: &Value,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let tgt = if let Some(tgt) = tgt {
            tgt
        } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };

        match value {
            Value::Nil => self.code.push(Insc::MakeBoolConst(false, tgt)),
            Value::Bool(value) => self.code.push(Insc::MakeBoolConst(*value, tgt)),
            Value::Char(value) => self.code.push(Insc::MakeIntConst(*value as i64, tgt)),
            Value::Uint(_) => unreachable!(),
            Value::Int(value) => self.code.push(Insc::MakeIntConst(*value, tgt)),
            Value::Float(value) => self.code.push(Insc::MakeFloatConst(*value, tgt)),
            Value::Str(value) => {
                let const_id: i64 = analyse_result.data_collection.get(value.as_ref(), "ConstID")
                    .unwrap()
                    .clone()
                    .try_into()
                    .unwrap();
                let const_id = bitcast_i64_usize(const_id);
                self.code.push(Insc::LoadConst(const_id, tgt));
            },
            Value::Pair(pair) => {
                let lhs = self.compile_value(&pair.0, analyse_result, None);
                let rhs = self.compile_value(&pair.1, analyse_result, None);
                let cons_func_id: i64 = analyse_result.global_data_map.get("BuiltinConsFuncID")
                    .expect("the `cons` function should be built into the program")
                    .clone()
                    .try_into()
                    .unwrap();
                let cons_func_id = bitcast_i64_usize(cons_func_id);
                self.code.push(Insc::Call(
                    cons_func_id,
                    unsafe { self.slice_arena.unsafe_make(&[lhs, rhs]) },
                    unsafe { self.slice_arena.unsafe_make(&[tgt]) }
                ));
            },
            _ => unreachable!()
        }

        tgt
    }

    fn compile_var(
        &mut self,
        var: Handle<Symbol>,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let mut g = guard2!(
            var.1,
            "compile variable `{}`",
            var.0.as_str()
        );
        let var_ref: Vec<GValue> = analyse_result.data_collection.get(var.as_ref(), "Ref")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        match <GValue as TryInto<String>>::try_into(var_ref[0].clone()).unwrap().as_str() {
            "Variable" => {
                let is_capture: bool = var_ref[1].clone().try_into().unwrap();
                let var_id = var_ref[2].clone().try_into().unwrap();
                let mut var_id = bitcast_i64_usize(var_id);

                if !is_capture {
                    var_id = self.compiling_function_chain.last().unwrap()
                        .translate_local_id_to_address(var_id);
                }
                g.cancel();

                if let Some(tgt) = tgt {
                    self.code.push(Insc::Move(var_id, tgt));
                    tgt
                } else {
                    var_id
                }
            },
            "Function" => {
                let func_id = var_ref[1].clone().try_into().unwrap();
                let func_id = bitcast_i64_usize(func_id);

                let tgt = if let Some(tgt) = tgt {
                    tgt
                } else {
                    self.compiling_function_chain.last_mut().unwrap().allocate_temp()
                };
                g.cancel();

                self.convert_func_to_closure(func_id, analyse_result, tgt);
                tgt
            },
            "FFI" => {
                panic!("FFI functions are not first-class!")
            },
            _ => unreachable!()
        }
    }

    fn compile_var_for_fn_call(
        &mut self,
        var: Handle<Symbol>,
        analyse_result: &mut AnalyseResult,
    ) -> MantisGod<usize, usize, (bool, usize)> {
        let mut g = guard2!(
            var.1,
            "compile variable `{}` for function call",
            var.0.as_str()
        );
        let var_ref: Vec<GValue> = analyse_result.data_collection.get(var.as_ref(), "Ref")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        match <GValue as TryInto<String>>::try_into(var_ref[0].clone()).unwrap().as_str() {
            "Variable" => {
                let is_capture: bool = var_ref[1].clone().try_into().unwrap();
                let var_id = var_ref[2].clone().try_into().unwrap();
                let var_id = bitcast_i64_usize(var_id);
                g.cancel();
                MantisGod::Left(if is_capture {
                    var_id
                } else {
                    self.compiling_function_chain.last().unwrap()
                        .translate_local_id_to_address(var_id)
                })
            },
            "Function" => {
                let func_id = var_ref[1].clone().try_into().unwrap();
                let func_id = bitcast_i64_usize(func_id);

                let captures: Vec<GValue> = analyse_result.functions
                    .get_raw_key(func_id, "Captures")
                    .unwrap()
                    .clone()
                    .try_into()
                    .unwrap();
                if !captures.is_empty() {
                    let tgt = self.compiling_function_chain.last_mut().unwrap().allocate_temp();
                    self.convert_func_to_closure(func_id, analyse_result, tgt);
                    g.cancel();
                    MantisGod::Left(tgt)
                } else {
                    g.cancel();
                    MantisGod::Middle(func_id)
                }
            },
            "FFI" => {
                let is_async = var_ref[1].clone().try_into().unwrap();
                let ffi_func_id = var_ref[2].clone().try_into().unwrap();
                let ffi_func_id = bitcast_i64_usize(ffi_func_id);
                g.cancel();
                MantisGod::Right((is_async, ffi_func_id))
            },
            _ => unreachable!()
        }
    }

    fn compile_lambda(
        &mut self,
        lambda: Handle<Function>,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let mut g = guard2!(
            lambda.pos,
            "compile lambda `{}@{:x}`",
            self.format_function_chain(),
            lambda.as_ref() as *const _ as usize
        );
        self.func_queue.push_back((self.compiling_function_names.clone(), lambda.clone()));

        let func_id = analyse_result.data_collection.get(lambda.as_ref(), "FunctionID")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let func_id = bitcast_i64_usize(func_id);

        let tgt = if let Some(tgt) = tgt {
            tgt
        } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };
        self.convert_func_to_closure(func_id, analyse_result, tgt);
        g.cancel();
        tgt
    }

    fn compile_let(
        &mut self,
        let_item: Handle<Let>,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let mut g = guard2!(
            let_item.pos,
            "compile let @{:x}",
            let_item.as_ref() as *const _ as usize
        );
        for bind in let_item.binds.iter() {
            let mut g = guard2!(
                bind.0.1,
                "compile let bind `{}`",
                bind.0.0.as_str()
            );
            let var_id = analyse_result.data_collection.get(bind.0.0.as_ref(), "VarID")
                .unwrap()
                .clone()
                .try_into()
                .unwrap();
            let var_id = bitcast_i64_usize(var_id);
            let var_id = self.compiling_function_chain.last().unwrap()
                .translate_local_id_to_address(var_id);
            self.compile_expr(&bind.1, analyse_result, Some(var_id));
            g.cancel();
        }

        let body_ret = self.compile_stmt_list(&let_item.body, analyse_result);
        let tgt = if let Some(tgt) = tgt {
            tgt
        } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };

        if let Some(ret_pos) = body_ret {
            self.code.push(Insc::Move(ret_pos, tgt));
        } else {
            self.code.push(Insc::MakeBoolConst(false, tgt));
        }
        g.cancel();
        tgt
    }

    fn compile_set(
        &mut self,
        set: Handle<Set>,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let mut g = guard2!(
            set.pos,
            "compile set var `{}`",
            set.name.0.as_str()
        );
        let var_id = analyse_result.data_collection.get(set.as_ref(), "VarID")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let var_id = bitcast_i64_usize(var_id);
        let var_id = self.compiling_function_chain.last().unwrap()
            .translate_local_id_to_address(var_id);
        self.compile_expr(&set.value, analyse_result, Some(var_id));

        g.cancel();
        if let Some(tgt) = tgt {
            self.code.push(Insc::Move(var_id, tgt));
            tgt
        } else {
            var_id
        }
    }

    fn compile_cond(
        &mut self,
        cond: Handle<Cond>,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let mut g = guard2!(
            cond.pos,
            "compile cond @{:x}",
            cond.as_ref() as *const _ as usize
        );

        let tgt = if let Some(tgt) = tgt { tgt } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };

        let mut last_condition_fail_jump_idx: Option<usize> = None;
        let mut jump_to_end_idx = Vec::new();
        for pair in cond.pairs.iter() {
            if let Some(idx) = last_condition_fail_jump_idx {
                let code_len = self.code.len();
                if let Insc::JumpIfFalse(_, dest) = &mut self.code[idx] {
                    *dest = code_len;
                } else {
                    unreachable!()
                }
            }

            let condition = self.compile_expr(&pair.0, analyse_result, None);
            let code_idx = self.code.len();
            last_condition_fail_jump_idx = Some(code_idx);

            self.code.push(Insc::JumpIfFalse(condition, 0));
            self.compile_expr(&pair.1, analyse_result, Some(tgt));
            let code_idx = self.code.len();
            jump_to_end_idx.push(code_idx);
            self.code.push(Insc::Jump(0));
        }

        if let Some(idx) = last_condition_fail_jump_idx {
            let code_len = self.code.len();
            if let Insc::JumpIfFalse(_, dest) = &mut self.code[idx] {
                *dest = code_len;
            } else {
                unreachable!()
            }
        }

        if let Some(else_branch) = cond.other.as_ref() {
            self.compile_expr(else_branch, analyse_result, Some(tgt));
        } else {
            self.code.push(Insc::CreateObject(0));
            self.code.push(Insc::Raise(0));
        }

        let code_len = self.code.len();
        for idx in jump_to_end_idx {
            if let Insc::Jump(dest) = &mut self.code[idx]{
                *dest = code_len;
            }
        }

        g.cancel();
        tgt
    }

    fn compile_call(
        &mut self,
        call: Handle<Call>,
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let mut g = if let Some(pos) = call.0[0].location() {
            guard2!(
                pos,
                "compile call @{:x}",
                call.as_ref() as *const _ as usize
            )
        } else {
            guard!("compile call @{:x}", call.as_ref() as *const _ as usize)
        };

        let tgt = if let Some(tgt) = tgt { tgt } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };

        if let Expr::Variable(sym) = &call.0[0] {
            if let Some(result) = self.try_compile_short_circuit_builtin(
                sym.0.as_str(),
                &call.0[1..],
                analyse_result,
                Some(tgt)
            ) {
                g.cancel();
                return result;
            }
        }

        let mut args = Vec::new();
        for arg in call.as_ref().0.iter().skip(1) {
            args.push(self.compile_expr(arg, analyse_result, None));
        }

        if let Expr::Variable(sym) = &call.0[0] {
            if let Some(result) = self.try_compile_builtin(sym.0.as_str(), &args, Some(tgt)) {
                g.cancel();
                return result;
            }
        }

        let called_func = self.compile_expr_for_fn_call(&call.0[0], analyse_result);

        match called_func {
            MantisGod::Left(var_id) => {
                let closure_tyck_info = unsafe {
                    let closure_vt = self.create_closure_vt(call.0.len() - 1);
                    self.tyck_info_pool.create_container_type(
                        TypeId::of::<Closure>(),
                        closure_vt.as_ref()
                            .tyck_info
                            .as_ref()
                            .params.as_ref()
                    )
                };

                // TODO support va-args
                self.code.push(Insc::TypeCheck(var_id, closure_tyck_info));
                self.code.push(Insc::CallPtr(var_id, unsafe {
                    self.slice_arena.unsafe_make(&args)
                }, unsafe {
                    self.slice_arena.unsafe_make(&[tgt])
                }));
            },
            MantisGod::Middle(func_id) => {
                let func_name: String = analyse_result.functions.get_raw_key(func_id, "FunctionName")
                    .unwrap()
                    .clone()
                    .try_into()
                    .unwrap();
                let mut g = guard!(
                    "compile call to function `{}` (func_id = {})",
                    func_name,
                    func_id
                );

                let p_args: Vec<_> = analyse_result.functions.get_raw_key(func_id, "ParamVarIDs")
                    .unwrap()
                    .clone()
                    .try_into()
                    .unwrap();
                let arg_count = p_args.len();

                // TODO support va-args
                assert_eq!(arg_count, call.0.len() - 1,
                           "expected {} args, got {}",
                           arg_count, call.0.len() - 1);

                self.code.push(Insc::Call(func_id, unsafe {
                    self.slice_arena.unsafe_make(&args)
                }, unsafe {
                    self.slice_arena.unsafe_make(&[tgt])
                }));
                g.cancel();
            },
            MantisGod::Right((is_async, ffi_func_id)) => {
                let func_name = if !is_async {
                    analyse_result.ffi_function_map.get(&ffi_func_id).unwrap()
                } else {
                    analyse_result.async_ffi_function_map.get(&ffi_func_id).unwrap()
                };

                let mut g = guard!(
                    "compile call to {} FFI function `{}` (func_id = {})",
                    if is_async { "async" } else { "sync" },
                    func_name,
                    ffi_func_id
                );

                let (arg_count, signature) = if !is_async {
                    let signature = self.ffi_funcs[ffi_func_id]
                        .signature(&mut self.tyck_info_pool);
                    (signature.param_options.len(), signature)
                } else {
                    let signature = self.async_ffi_funcs[ffi_func_id]
                        .signature(&mut self.tyck_info_pool);
                    (signature.param_options.len(), signature)
                };

                assert_eq!(arg_count, call.0.len() - 1,
                           "expected {} args, got {}",
                           arg_count, call.0.len() - 1);

                for i in 0..arg_count {
                    self.code.push(Insc::TypeCheck(args[i], unsafe {
                        if let TyckInfo::Function(func) = signature.func_type.as_ref() {
                            func.params.as_ref()[i].clone()
                        } else {
                            unreachable!()
                        }
                    }));
                }

                if !is_async {
                    self.code.push(Insc::FFICallRtlc(
                        ffi_func_id,
                        unsafe { self.slice_arena.unsafe_make(&args) },
                        unsafe { self.slice_arena.unsafe_make(&[tgt]) }
                    ));
                } else {
                    let tmp = self.compiling_function_chain.last_mut().unwrap().allocate_temp();
                    self.code.push(Insc::FFICallAsync(
                        ffi_func_id,
                        unsafe { self.slice_arena.unsafe_make(&args) },
                        tmp
                    ));
                    self.code.push(Insc::Await(tmp, unsafe {
                        self.slice_arena.unsafe_make(&[tgt])
                    }));
                }

                g.cancel();
            }
        }

        g.cancel();
        tgt
    }

    fn try_compile_builtin(
        &mut self,
        op: &str,
        args: &[usize],
        tgt: Option<usize>
    ) -> Option<usize> {
        let tgt = if let Some(tgt) = tgt { tgt } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };

        Some(match op {
            "display" => {
                assert!(args.len() <= 32, "FFI function only supports up to 32 args");
                self.code.push(Insc::FFICallRtlc(
                    0,
                    unsafe { self.slice_arena.unsafe_make(&args) },
                    unsafe { self.slice_arena.unsafe_make(&[tgt]) }
                ));
                tgt
            },
            "=" => {
                assert_eq!(args.len(), 2, "`=` expects 2 arguments");
                self.code.push(Insc::EqAny(args[0], args[1], tgt));
                tgt
            },
            "+" => {
                assert!(args.len() >= 2, "`+` expects at least 2 arguments");
                if args.len() == 2 {
                    self.code.push(Insc::AddAny(args[0], args[1], tgt));
                } else {
                    self.code.push(Insc::Move(args[0], tgt));
                    for i in 1..args.len() {
                        self.code.push(Insc::AddAny(tgt, args[i], tgt));
                    }
                }
                tgt
            },
            "~" => {
                assert!(args.len() >= 2, "`~` expects at least 2 arguments");
                let string_type = self.tyck_info_pool.get_string_type();
                for i in 0..args.len() {
                    self.code.push(Insc::TypeCheck(args[i], string_type));
                }

                self.code.push(Insc::StrConcat(
                    unsafe { self.slice_arena.unsafe_make(&args) },
                    tgt
                ));
                tgt
            },
            "-" => {
                assert_eq!(args.len(), 2, "`-` expects 2 arguments");
                self.code.push(Insc::SubAny(args[0], args[1], tgt));
                tgt
            },
            "*" => {
                assert!(args.len() >= 2, "`*` expects at least 2 arguments");
                if args.len() == 2 {
                    self.code.push(Insc::MulAny(args[0], args[1], tgt));
                } else {
                    self.code.push(Insc::Move(args[0], tgt));
                    for i in 1..args.len() {
                        self.code.push(Insc::MulAny(tgt, args[i], tgt));
                    }
                }
                tgt
            },
            "/" => {
                assert_eq!(args.len(), 2, "`/` expects 2 arguments");
                self.code.push(Insc::DivAny(args[0], args[1], tgt));
                tgt
            },
            "%" => {
                assert_eq!(args.len(), 2, "`%` expects 2 arguments");
                self.code.push(Insc::ModAny(args[0], args[1], tgt));
                tgt
            },
            ">" => {
                assert_eq!(args.len(), 2, "`>` expects 2 arguments");
                self.code.push(Insc::LtAny(args[1], args[0], tgt));
                tgt
            },
            "<" => {
                assert_eq!(args.len(), 2, "`<` expects 2 arguments");
                self.code.push(Insc::LtAny(args[0], args[1], tgt));
                tgt
            },
            ">=" => {
                assert_eq!(args.len(), 2, "`>=` expects 2 arguments");
                self.code.push(Insc::LeAny(args[1], args[0], tgt));
                tgt
            },
            "<=" => {
                assert_eq!(args.len(), 2, "`<=` expects 2 arguments");
                self.code.push(Insc::LeAny(args[0], args[1], tgt));
                tgt
            },
            "!=" => {
                assert_eq!(args.len(), 2, "`!=` expects 2 arguments");
                self.code.push(Insc::NeAny(args[0], args[1], tgt));
                tgt
            },
            "not" => {
                assert_eq!(args.len(), 1, "`not` expects 1 argument");
                self.code.push(Insc::NotAny(args[0], tgt));
                tgt
            },
            "raise" => {
                assert_eq!(args.len(), 1, "`raise` expects 1 argument");
                let string_type = self.tyck_info_pool.get_string_type();
                self.code.push(Insc::TypeCheck(args[0], string_type));
                self.code.push(Insc::Raise(args[0]));
                tgt
            },
            "begin" | "unused" | "pass" => {
                self.code.push(Insc::MakeBoolConst(false, tgt));
                tgt
            },
            "string-length" | "strlen" => {
                assert_eq!(args.len(), 1, "`string-length` or `strlen` expects 1 argument");
                let string_type = self.tyck_info_pool.get_string_type();
                self.code.push(Insc::TypeCheck(args[0], string_type));
                self.code.push(Insc::OwnershipInfoCheck(args[0], OWN_INFO_READ_MASK));
                self.code.push(Insc::StrLen(args[0], tgt));
                tgt
            },
            "string-equals?" | "strcmp" => {
                assert_eq!(args.len(), 2, "`string-equals?` or `strcmp` expects 2 arguments");
                let string_type = self.tyck_info_pool.get_string_type();
                self.code.push(Insc::TypeCheck(args[0], string_type));
                self.code.push(Insc::TypeCheck(args[1], string_type));
                self.code.push(Insc::OwnershipInfoCheck(args[0], OWN_INFO_READ_MASK));
                self.code.push(Insc::OwnershipInfoCheck(args[1], OWN_INFO_READ_MASK));
                self.code.push(Insc::StrEquals(args[0], args[1], tgt));
                tgt
            },
            "vector" => {
                let vector_vt = self.create_vector_vt();
                self.code.push(Insc::CreateContainer(vec_ctor, vector_vt, tgt));
                for arg in args {
                    self.code.push(Insc::VecPush(tgt, *arg));
                }
                tgt
            },
            "vector-length" => {
                assert_eq!(args.len(), 1, "`vector-length` expects 1 argument");
                let any_type = self.tyck_info_pool.get_any_type();
                let vec_type = self.tyck_info_pool.create_container_type(TypeId::of::<VMGenericVec>(), &[any_type]);
                self.code.push(Insc::TypeCheck(args[0], vec_type));
                self.code.push(Insc::OwnershipInfoCheck(args[0], OWN_INFO_READ_MASK));
                self.code.push(Insc::VecLen(args[0], tgt));
                tgt
            },
            "vector-ref" => {
                assert_eq!(args.len(), 2, "`vector-ref` expects 2 arguments");
                let any_type = self.tyck_info_pool.get_any_type();
                let int_type = self.tyck_info_pool.get_int_type();
                let vec_type = self.tyck_info_pool.create_container_type(TypeId::of::<VMGenericVec>(), &[any_type]);
                self.code.push(Insc::TypeCheck(args[0], vec_type));
                self.code.push(Insc::TypeCheck(args[1], int_type));
                self.code.push(Insc::OwnershipInfoCheck(args[0], OWN_INFO_READ_MASK));
                self.code.push(Insc::VecIndex(args[0], args[1], tgt));
                tgt
            },
            "vector-push!" => {
                assert_eq!(args.len(), 2, "`vector-push!` expects 2 arguments");
                let any_type = self.tyck_info_pool.get_any_type();
                let vec_type = self.tyck_info_pool.create_container_type(TypeId::of::<VMGenericVec>(), &[any_type]);
                self.code.push(Insc::TypeCheck(args[0], vec_type));
                self.code.push(Insc::OwnershipInfoCheck(args[0], OWN_INFO_WRITE_MASK));
                self.code.push(Insc::VecPush(args[0], args[1]));
                self.code.push(Insc::MakeBoolConst(false, tgt));
                tgt
            },
            "vector-set!" => {
                assert_eq!(args.len(), 3, "`vector-set!` expects 3 arguments");
                let any_type = self.tyck_info_pool.get_any_type();
                let int_type = self.tyck_info_pool.get_int_type();
                let vec_type = self.tyck_info_pool.create_container_type(TypeId::of::<VMGenericVec>(), &[any_type]);
                self.code.push(Insc::TypeCheck(args[0], vec_type));
                self.code.push(Insc::TypeCheck(args[1], int_type));
                self.code.push(Insc::OwnershipInfoCheck(args[0], OWN_INFO_WRITE_MASK));
                self.code.push(Insc::VecIndexPut(args[0], args[1], args[2]));
                self.code.push(Insc::MakeBoolConst(false, tgt));
                tgt
            },
            "object" => {
                assert_eq!(args.len(), 0, "`object` expects 0 arguments");
                self.code.push(Insc::CreateObject(tgt));
                tgt
            },
            "object-get" => {
                assert_eq!(args.len(), 2, "`object-get` expects 2 arguments");
                let obj_type = self.tyck_info_pool.get_object_type();
                let string_type = self.tyck_info_pool.get_string_type();
                self.code.push(Insc::TypeCheck(args[0], obj_type));
                self.code.push(Insc::TypeCheck(args[1], string_type));
                self.code.push(Insc::OwnershipInfoCheck(args[0], OWN_INFO_READ_MASK));
                self.code.push(Insc::OwnershipInfoCheck(args[1], OWN_INFO_READ_MASK));
                self.code.push(Insc::ObjectGetDyn(args[0], args[1], tgt));
                tgt
            },
            "object-set!" => {
                assert_eq!(args.len(), 3, "`object-set!` expects 3 arguments");
                let obj_type = self.tyck_info_pool.get_object_type();
                let string_type = self.tyck_info_pool.get_string_type();
                self.code.push(Insc::TypeCheck(args[0], obj_type));
                self.code.push(Insc::TypeCheck(args[1], string_type));
                self.code.push(Insc::OwnershipInfoCheck(args[0], OWN_INFO_WRITE_MASK));
                self.code.push(Insc::OwnershipInfoCheck(args[1], OWN_INFO_READ_MASK));
                self.code.push(Insc::ObjectPutDyn(args[0], args[1], args[2]));
                self.code.push(Insc::MakeBoolConst(false, tgt));
                tgt
            },
            _ => return None
        })
    }

    fn try_compile_short_circuit_builtin(
        &mut self,
        op: &str,
        args: &[Expr],
        analyse_result: &mut AnalyseResult,
        tgt: Option<usize>
    ) -> Option<usize> {
        let tgt = if let Some(tgt) = tgt { tgt } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };

        Some(match op {
            "loop" => {
                let loop_start = self.code.len();
                self.compiling_loops.push(LoopContext::new(loop_start));
                for arg in args {
                    self.compile_expr(arg, analyse_result, None);
                }
                self.code.push(Insc::Jump(loop_start));
                let loop_end = self.code.len();
                let loop_ctx = self.compiling_loops.pop().unwrap();

                for pending_break in loop_ctx.pending_breaks.iter() {
                    self.code[*pending_break] = Insc::Jump(loop_end);
                }

                self.code.push(Insc::MakeBoolConst(false, tgt));
                tgt
            },
            "break" => {
                let loop_ctx = self.compiling_loops.last_mut().unwrap();
                loop_ctx.pending_breaks.push(self.code.len());
                self.code.push(Insc::MakeBoolConst(false, tgt));
                tgt
            },
            "continue" => {
                let loop_ctx = self.compiling_loops.last_mut()
                    .expect("`continue` outside of loop");
                self.code.push(Insc::Jump(loop_ctx.loop_start_addr));
                tgt
            },
            "if" => {
                assert!(args.len() == 3 || args.len() == 2, "`if` expects 2 or 3 arguments");
                let cond = self.compile_expr(&args[0], analyse_result, None);
                let then_addr = self.code.len();
                self.code.push(Insc::JumpIfFalse(0, 0));
                self.compile_expr(&args[1], analyse_result, Some(tgt));
                let then_done_addr = self.code.len();
                self.code.push(Insc::Jump(0));
                let else_addr = self.code.len();
                if args.len() == 3 {
                    self.compile_expr(&args[2], analyse_result, Some(tgt));
                } else {
                    self.code.push(Insc::MakeBoolConst(false, tgt));
                }
                let done_addr = self.code.len();

                self.code[then_addr] = Insc::JumpIfFalse(cond, else_addr);
                self.code[then_done_addr] = Insc::Jump(done_addr);

                tgt
            },
            "spawn" => {
                if let MantisGod::Middle(func_id) = self.compile_expr_for_fn_call(&args[0], analyse_result) {
                    let param_var_ids: Vec<GValue> = analyse_result.functions
                        .get_raw_key(func_id, "ParamVarIDs")
                        .unwrap()
                        .clone()
                        .try_into()
                        .unwrap();
                    let param_count = param_var_ids.len();
                    assert_eq!(param_count, args.len() - 1,
                               "`spawn` expects the same number of arguments as the function it is spawning");
                    let mut spawn_args = Vec::new();
                    for arg in args.iter().skip(1) {
                        spawn_args.push(self.compile_expr(arg, analyse_result, None));
                    }

                    self.code.push(Insc::Spawn(func_id, unsafe { self.slice_arena.unsafe_make(&spawn_args) }));
                    self.code.push(Insc::Await(0, unsafe { self.slice_arena.unsafe_make(&[tgt]) }));
                    tgt
                } else {
                    panic!("`spawn` expects a normal function as its first argument");
                }
            },
            "and" => {
                assert!(args.len() >= 2, "`and` requires at least two arguments");
                let mut jump_to_fail_idx = Vec::new();
                for arg in args {
                    let tmp = self.compile_expr(arg, analyse_result, None);
                    jump_to_fail_idx.push(self.code.len());
                    self.code.push(Insc::JumpIfFalse(tmp, 0));
                }
                self.code.push(Insc::MakeBoolConst(true, tgt));
                let jump_to_next_idx = self.code.len();
                self.code.push(Insc::Jump(0));

                let code_len = self.code.len();
                for i in 0..args.len() {
                    if let Insc::JumpIfFalse(_, dest) = &mut self.code[jump_to_fail_idx[i]] {
                        *dest = code_len;
                    } else {
                        unreachable!();
                    }
                }
                self.code.push(Insc::MakeBoolConst(false, tgt));
                let code_len = self.code.len();
                if let Insc::Jump(dest) = &mut self.code[jump_to_next_idx] {
                    *dest = code_len;
                } else {
                    unreachable!();
                }

                tgt
            },
            "or" => {
                assert!(args.len() >= 2, "`or` requires at least two arguments");
                let mut jump_to_next_idx = Vec::new();
                for arg in args {
                    let tmp = self.compile_expr(arg, analyse_result, None);
                    jump_to_next_idx.push(self.code.len());
                    self.code.push(Insc::JumpIfTrue(tmp, 0));
                }
                self.code.push(Insc::MakeBoolConst(false, tgt));
                let jump_to_fail_idx = self.code.len();
                self.code.push(Insc::Jump(0));

                let code_len = self.code.len();
                for i in 0..args.len() {
                    if let Insc::JumpIfTrue(_, dest) = &mut self.code[jump_to_next_idx[i]] {
                        *dest = code_len;
                    } else {
                        unreachable!();
                    }
                }
                self.code.push(Insc::MakeBoolConst(true, tgt));
                let code_len = self.code.len();
                if let Insc::Jump(dest) = &mut self.code[jump_to_fail_idx] {
                    *dest = code_len;
                } else {
                    unreachable!();
                }

                tgt
            },
            _ => return None
        })
    }
}

impl CompileContext {
    fn convert_func_to_closure(&mut self, func_id: usize, analyse_result: &mut AnalyseResult, tgt: usize) {
        let params: Vec<GValue> = analyse_result.functions
            .get_raw_key(func_id, "ParamVarIDs")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let captures: Vec<GValue> = analyse_result.functions
            .get_raw_key(func_id, "Captures")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let captures: Vec<_> = captures.into_iter()
            .map(|item: GValue| {
                let item: Vec<GValue> = item.try_into().unwrap();
                (
                    item[0].clone().try_into().unwrap(),
                    bitcast_i64_usize(item[1].clone().try_into().unwrap())
                )
            })
            .collect();

        let mut captured_item_address = Vec::new();
        for (is_another_capture, item_id) in captures {
            let item_id = if is_another_capture {
                item_id
            } else {
                self.compiling_function_chain.last().unwrap()
                    .translate_local_id_to_address(item_id)
            };
            captured_item_address.push(item_id);
        }
        let captured_item_address = unsafe {
            self.slice_arena.unsafe_make(&captured_item_address)
        };

        let vt = self.create_closure_vt(params.len());
        self.code.push(Insc::CreateClosure(func_id, captured_item_address, vt, tgt));
    }

    fn create_closure_vt(&mut self, closure_arg_count: usize) -> NonNull<GenericTypeVT> {
        let closure_arg_types: Vec<NonNull<TyckInfo>> = (0..closure_arg_count).into_iter()
            .map(|_| self.tyck_info_pool.get_any_type())
            .collect();

        let korobka = Korobka::new(pr47::builtins::closure::create_closure_vt(
            &mut self.tyck_info_pool,
            &closure_arg_types
        ));

        let ret = korobka.as_nonnull();
        self.vt_pool.push(korobka);
        ret
    }

    fn create_vector_vt(&mut self) -> NonNull<GenericTypeVT> {
        let any_type = self.tyck_info_pool.get_any_type();
        let korobka = Korobka::new(pr47::builtins::vec::create_vm_vec_vt(
            &mut self.tyck_info_pool,
            any_type
        ));

        let ret = korobka.as_nonnull();
        self.vt_pool.push(korobka);
        ret
    }
}
