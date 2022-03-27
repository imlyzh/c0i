use std::collections::HashMap;
use std::convert::TryInto;
use std::ptr::NonNull;
use pr47::data::generic::GenericTypeVT;
use pr47::data::tyck::{TyckInfo, TyckInfoPool};
use pr47::vm::al31f::insc::Insc;
use sexpr_ir::gast::Handle;
use sexpr_ir::gast::symbol::Symbol;
use xjbutil::boxed_slice;
use xjbutil::korobka::Korobka;
use xjbutil::slice_arena::SliceArena;
use crate::ast::{Expr, Function, TopLevel};
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

impl CompilingFunction {
    fn allocate_temp(&mut self) -> usize {
        let ret = self.local_count + self.capture_count;
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

        for stmt in func.body.iter() {
            self.compile_stmt(stmt, analyse_result, None);
        }

        let compiling_function = self.compiling_function_chain.pop().unwrap();
        self.functions.insert(func_id, CompiledFunction {
            start_addr,
            arg_count: param_var_ids.len(),
            ret_count: 1,
            stack_size: compiling_function.local_count,
            param_tyck_info: boxed_slice![],
            exc_handlers: None
        });
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
            Expr::Lambda(lambda) => self.compile_lambda(lambda, analyse_result, tgt),
            Expr::Let(let_item) => self.compile_let(let_item, analyse_result, tgt),
            Expr::Set(set) => self.compile_set(set, analyse_result, tgt),
            Expr::Cond(cond) => self.compile_cond(cond, analyse_result, tgt),
            Expr::FunctionCall(call) => self.compile_call(call, analyse_result, tgt),
        }
    }

    fn compile_expr_for_fn_call(
        &mut self,
        _expr: &Expr,
        _analyse_result: &AnalyseResult
    ) -> MantisGod<usize, usize, usize> {
        todo!()
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
                let var_id = var_ref[1].clone().try_into().unwrap();
                let mut var_id = bitcast_i64_usize(var_id);
                let is_capture: bool = var_ref[2].clone().try_into().unwrap();

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

                let tgt = if let Some(tgt) = tgt {
                    tgt
                } else {
                    self.compiling_function_chain.last_mut().unwrap().allocate_temp()
                };

                let vt = self.create_closure_vt(params.len());
                let vt_ptr = vt.as_nonnull();
                self.vt_pool.push(vt);

                self.code.push(Insc::CreateClosure(func_id, captured_item_address, vt_ptr, tgt));
                tgt
            },
            "FFI" => {
                panic!("FFI functions are not first-class!")
            },
            _ => unreachable!()
        }
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
    fn create_closure_vt(&mut self, closure_arg_count: usize) -> Korobka<GenericTypeVT> {
        let closure_arg_types: Vec<NonNull<TyckInfo>> = (0..closure_arg_count).into_iter()
            .map(|_| self.tyck_info_pool.get_any_type())
            .collect();

        Korobka::new(pr47::builtins::closure::create_closure_vt(
            &mut self.tyck_info_pool,
            &closure_arg_types
        ))
    }
}
