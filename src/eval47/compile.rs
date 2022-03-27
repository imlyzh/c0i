use std::any::TypeId;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ptr::NonNull;
use pr47::builtins::closure::Closure;
use pr47::data::generic::GenericTypeVT;
use pr47::data::tyck::{TyckInfo, TyckInfoPool};
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
    compiling_function_names: Vec<String>
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
            compiling_function_names: Vec::new()
        };
        context
    }

    pub fn compile(
        mut self,
        ast: &[TopLevel],
        analyse_result: &AnalyseResult
    ) -> CompileResult {
        eprintln!("Compiling to bytecode");

        for piece in ast {
            self.compile_top_level(piece, analyse_result);
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

impl CompileContext {
    fn compile_top_level(&mut self, top_level: &TopLevel, analyse_result: &AnalyseResult) {
        if let TopLevel::Function(func_handle) = top_level {
            self.compile_function(func_handle.clone(), analyse_result);
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
        let captures: Vec<GValue> = analyse_result.data_collection
            .get(func.as_ref(), "Captures")
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
            self.code.push(Insc::MakeNull(ret_pos));
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
    }

    fn compile_stmt_list(
        &mut self,
        stmt_list: &[TopLevel],
        analyse_result: &AnalyseResult
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
        analyse_result: &AnalyseResult,
        tgt: Option<usize>
    ) -> Option<usize> {
        match stmt {
            TopLevel::Function(func_handle) => {
                self.compile_function(func_handle.clone(), analyse_result);
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
        analyse_result: &AnalyseResult,
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
        analyse_result: &AnalyseResult
    ) -> MantisGod<usize, usize, (bool, usize)> {
        match expr {
            Expr::Variable(var) => self.compile_var_for_fn_call(var.clone(), analyse_result),
            _ => MantisGod::Left(self.compile_expr(expr, analyse_result, None))
        }
    }

    fn compile_value(
        &mut self,
        value: &Value,
        analyse_result: &AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let tgt = if let Some(tgt) = tgt {
            tgt
        } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };

        match value {
            Value::Nil => self.code.push(Insc::MakeNull(tgt)),
            Value::Bool(value) => self.code.push(Insc::MakeBoolConst(*value, tgt)),
            Value::Char(value) => self.code.push(Insc::MakeCharConst(*value, tgt)),
            Value::Uint(_) => unreachable!(),
            Value::Int(value) => self.code.push(Insc::MakeIntConst(*value, tgt)),
            Value::Float(value) => self.code.push(Insc::MakeFloatConst(*value, tgt)),
            Value::Str(value) => {
                let const_id: i64 = analyse_result.data_collection.get(value, "ConstID")
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
        analyse_result: &AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
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

                if is_capture {
                    var_id = self.compiling_function_chain.last().unwrap()
                        .translate_local_id_to_address(var_id);
                }

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
        analyse_result: &AnalyseResult,
    ) -> MantisGod<usize, usize, (bool, usize)> {
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
                MantisGod::Middle(func_id)
            },
            "FFI" => {
                let is_async = var_ref[1].clone().try_into().unwrap();
                let ffi_func_id = var_ref[2].clone().try_into().unwrap();
                let ffi_func_id = bitcast_i64_usize(ffi_func_id);
                MantisGod::Right((is_async, ffi_func_id))
            },
            _ => unreachable!()
        }
    }

    fn compile_lambda(
        &mut self,
        lambda: Handle<Function>,
        analyse_result: &AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        self.compile_function(lambda.clone(), analyse_result);
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
        tgt
    }

    fn compile_let(
        &mut self,
        let_item: Handle<Let>,
        analyse_result: &AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        for bind in let_item.binds.iter() {
            let var_id = analyse_result.data_collection.get(bind.0.as_ref(), "VarID")
                .unwrap()
                .clone()
                .try_into()
                .unwrap();
            let var_id = bitcast_i64_usize(var_id);
            let var_id = self.compiling_function_chain.last().unwrap()
                .translate_local_id_to_address(var_id);
            self.compile_expr(&bind.1, analyse_result, Some(var_id));
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
            self.code.push(Insc::MakeNull(tgt));
        }
        tgt
    }

    fn compile_set(
        &mut self,
        set: Handle<Set>,
        analyse_result: &AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let var_id = analyse_result.data_collection.get(set.as_ref(), "VarID")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
        let var_id = bitcast_i64_usize(var_id);
        let var_id = self.compiling_function_chain.last().unwrap()
            .translate_local_id_to_address(var_id);
        self.compile_expr(&set.value, analyse_result, Some(var_id));

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
        analyse_result: &AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
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
            self.code.push(Insc::MakeIntConst(1145141919810, 0));
            self.code.push(Insc::Raise(0));
        }

        let code_len = self.code.len();
        for idx in jump_to_end_idx {
            if let Insc::Jump(dest) = &mut self.code[idx]{
                *dest = code_len;
            }
        }
        tgt
    }

    fn compile_call(
        &mut self,
        call: Handle<Call>,
        analyse_result: &AnalyseResult,
        tgt: Option<usize>
    ) -> usize {
        let tgt = if let Some(tgt) = tgt { tgt } else {
            self.compiling_function_chain.last_mut().unwrap().allocate_temp()
        };
        let mut args = Vec::new();
        for arg in call.as_ref().0.iter().skip(1) {
            args.push(self.compile_expr(arg, analyse_result, None));
        }

        if let Expr::Variable(sym) = &call.0[0] {
            match sym.0.as_str() {
                "=" => {
                    assert_eq!(args.len(), 2);
                    self.code.push(Insc::EqAny(args[0], args[1], tgt));
                    return tgt;
                },
                _ => {}
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
                let arg_count = self.functions.get(&func_id)
                    .as_ref()
                    .unwrap()
                    .arg_count;
                // TODO support va-args
                assert_eq!(arg_count, call.0.len() - 1);
                self.code.push(Insc::Call(func_id, unsafe {
                    self.slice_arena.unsafe_make(&args)
                }, unsafe {
                    self.slice_arena.unsafe_make(&[tgt])
                }));
            },
            MantisGod::Right((is_async, ffi_func_id)) => {
                let (arg_count, signature) = if !is_async {
                    let signature = self.ffi_funcs[ffi_func_id]
                        .signature(&mut self.tyck_info_pool);
                    (signature.param_options.len(), signature)
                } else {
                    let signature = self.async_ffi_funcs[ffi_func_id]
                        .signature(&mut self.tyck_info_pool);
                    (signature.param_options.len(), signature)
                };

                for i in 0..arg_count {
                    self.code.push(Insc::TypeCheck(args[i], unsafe {
                        if let TyckInfo::Function(func) = signature.func_type.as_ref() {
                            func.params.as_ref()[i].clone()
                        } else {
                            unreachable!()
                        }
                    }));
                }

                assert_eq!(arg_count, call.0.len() - 1);
                if is_async {
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
            }
        }

        tgt
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

impl CompileContext {
    fn convert_func_to_closure(&mut self, func_id: usize, analyse_result: &AnalyseResult, tgt: usize) {
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
}
