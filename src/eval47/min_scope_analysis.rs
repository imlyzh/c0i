use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;

use pr47::data::tyck::TyckInfoPool;
use sexpr_ir::gast::Handle;

use crate::ast::{Expr, Function, TopLevel};
use crate::eval47::commons::{FFIAsyncFunction, FFIFunction, Signature};
use crate::eval47::data_map::{DataCollection, GValue};
use crate::eval47::util::{bitcast_usize_i64, clone_signature, MantisGod};
use crate::value::Value;

pub struct AnalyseContext {
    tyck_info_pool: TyckInfoPool,
    ffi_functions: HashMap<String, (FFIFunction, Signature)>,
    async_ffi_functions: HashMap<String, (FFIAsyncFunction, Signature)>,
}

impl AnalyseContext {
    pub fn new() -> Self {
        AnalyseContext {
            tyck_info_pool: TyckInfoPool::new(),
            ffi_functions: HashMap::new(),
            async_ffi_functions: HashMap::new()
        }
    }

    pub fn register_ffi(&mut self, name: impl Into<String>, ffi: FFIFunction) {
        let signature: Signature = ffi.signature(&mut self.tyck_info_pool);
        self.ffi_functions.insert(name.into(), (ffi, signature));
    }

    pub fn register_async_ffi(&mut self, name: impl Into<String>, ffi: FFIAsyncFunction) {
        let signature: Signature = ffi.signature(&mut self.tyck_info_pool);
        self.async_ffi_functions.insert(name.into(), (ffi, signature));
    }

    pub fn min_scope_analyse(&mut self, ast: &[TopLevel]) -> AnalyseResult {
        let mut result = AnalyseResult::new();
        let mut scope_chain = Some(Box::new(Scope::new(None)));

        self.analyze_global(&mut result, &mut scope_chain, ast);

        result
    }
}

impl AnalyseContext {
    fn analyze_global(
        &mut self,
        result: &mut AnalyseResult,
        scope_chain: &mut Option<Box<Scope>>,
        globals: &[TopLevel]
    ) {
        for global in globals {
            if let TopLevel::Function(func_handle) = global {
                let func_id = scope_chain.as_mut().unwrap().add_func(
                    func_handle.name
                        .as_ref()
                        .expect("Why the function name is empty anyway?")
                        .0
                        .as_str()
                );
                result.data_collection.insert(
                    func_handle.as_ref(),
                    "FunctionID",
                    bitcast_usize_i64(func_id)
                );
            } else {
                panic!("global variables or expressions are not supported by Pr47");
            }
        }

        for global in globals {
            if let TopLevel::Function(func_handle) = global {
                self.analyze_function(result, scope_chain, func_handle.clone())
            } else {
                unreachable!();
            }
        }
    }

    fn analyze_function(
        &mut self,
        result: &mut AnalyseResult,
        scope_chain: &mut Option<Box<Scope>>,
        func: Handle<Function>
    ) {
        *scope_chain = Some(Box::new(Scope::new_function_frame(scope_chain.take())));

        let mut var_ids = vec![];
        for param in func.params.iter() {
            let var_id = scope_chain.as_mut().unwrap().add_var(param.0.as_str());
            var_ids.push(bitcast_usize_i64(var_id));
        }
        if let Some(extend_param) = func.extend_params.as_ref() {
            let var_id = scope_chain.as_mut().unwrap().add_var(extend_param.0.as_str());
            var_ids.push(bitcast_usize_i64(var_id));
        }

        result.data_collection.insert(func.as_ref(), "ParamVarIDs", var_ids);

        for stmt in func.body.iter() {
            self.analyze_stmt(result, scope_chain, stmt);
        }

        *scope_chain = scope_chain.take().unwrap().parent;
    }

    fn analyze_stmt(
        &mut self,
        result: &mut AnalyseResult,
        scope_chain: &mut Option<Box<Scope>>,
        stmt: &TopLevel
    ) {
        match stmt.borrow() {
            TopLevel::Function(func_handle) => self.analyze_function(
                result, scope_chain, func_handle.clone()
            ),
            TopLevel::Bind(var_name, expr) => {
                let var_id = scope_chain.as_mut().unwrap().add_var(var_name.0.as_str());
                result.data_collection.insert(
                    stmt,
                    "VarID",
                    bitcast_usize_i64(var_id)
                );
                self.analyze_expr(result, scope_chain, expr);
            }
            TopLevel::Expr(expr) => self.analyze_expr(result, scope_chain, expr)
        }
    }

    fn analyze_expr(
        &mut self,
        result: &mut AnalyseResult,
        scope_chain: &mut Option<Box<Scope>>,
        expr: &Expr
    ) {
        match expr {
            Expr::Value(value) => self.analyze_value(result, scope_chain, value),
            Expr::Variable(_) => {}
            Expr::Lambda(_) => {}
            Expr::Let(_) => {}
            Expr::Set(_) => {}
            Expr::Cond(_) => {}
            Expr::FunctionCall(_) => {}
        }
    }

    fn analyze_value(
        &mut self,
        result: &mut AnalyseResult,
        scope_chain: &mut Option<Box<Scope>>,
        value: &Value
    ) {
        match value {
            Value::Uint(_) => panic!("Uint value is not supported by Pr47"),
            Value::Str(s) => {
                let const_id = result.global_consts.len();
                result.global_consts.push(GValue::String(s.as_str().to_string()));
                result.data_collection.insert(
                    value,
                    "ConstID",
                    bitcast_usize_i64(const_id)
                );
            },
            Value::Sym(_) => panic!("Sym value is not supported by Pr47"),
            Value::Pair(pair) => {
                self.analyze_value(result, scope_chain, &pair.0);
                self.analyze_value(result, scope_chain, &pair.1);
            },
            Value::Dict(_) => panic!("Not supported yet"),
            Value::Vec(_) => panic!("Not supported yet"),
            Value::Callable(_) => unreachable!(),
            _ => {}
        }
    }
}

pub struct AnalyseResult<'a> {
    pub data_collection: DataCollection,
    pub global_data_map: HashMap<String, GValue>,
    pub global_consts: Vec<GValue>,
    pub ffi_function_in_use: HashMap<String, (FFIFunction, Signature, usize)>,
    pub async_ffi_function_in_use: HashMap<String, (FFIAsyncFunction, Signature, usize)>,
    _phantom: PhantomData<&'a ()>
}

impl<'a> AnalyseResult<'a> {
    fn new() -> Self {
        Self {
            data_collection: DataCollection::new(),
            global_data_map: HashMap::new(),
            global_consts: Vec::new(),
            ffi_function_in_use: HashMap::new(),
            async_ffi_function_in_use: HashMap::new(),
            _phantom: PhantomData
        }
    }
}

struct FunctionFrame {
    register_alloc: usize,
    captures: HashMap<String, (
        usize, // capture id
        bool, // the captured item is another capture?
        usize, // captured item
    )>
}

struct Scope {
    parent: Option<Box<Scope>>,
    function_frame: Option<FunctionFrame>,

    variables: HashMap<String, (
        bool, // is capture?
        usize // item id
    )>,
    functions: HashMap<String, usize>,
    function_counter: usize
}

type LookupResult = MantisGod<
    (bool, usize), // variable
    usize, // script-defined function,
    (bool, usize) // FFI function
>;

type LookupContext<'a> = (
    &'a HashMap<String, (FFIFunction, Signature)>,
    &'a HashMap<String, (FFIAsyncFunction, Signature)>,
    &'a mut AnalyseResult<'a>
);

impl Scope {
    fn new(parent: Option<Box<Scope>>) -> Self {
        Self {
            parent,
            function_frame: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
            function_counter: 0
        }
    }

    fn new_function_frame(parent: Option<Box<Scope>>) -> Self {
        Self {
            parent,
            function_frame: Some(FunctionFrame {
                register_alloc: 0,
                captures: HashMap::new()
            }),
            variables: HashMap::new(),
            functions: HashMap::new(),
            function_counter: 0
        }
    }

    fn lookup(&mut self, ctx: &mut LookupContext, name: &str) -> Option<LookupResult> {
        if let Some(variable) = self.variables.get(name) {
            Some(MantisGod::Left(variable.clone()))
        } else if let Some((ffi_function, signature)) = ctx.0.get(name) {
            let func_id = if !ctx.2.ffi_function_in_use.contains_key(name) {
                let func_id = ctx.2.ffi_function_in_use.len();
                ctx.2.ffi_function_in_use.insert(
                    name.to_string(),
                    (
                        ffi_function.clone(),
                        clone_signature(signature),
                        func_id
                    )
                );
                func_id
            } else {
                ctx.2.ffi_function_in_use.get(name).unwrap().2
            };
            Some(MantisGod::Right((false, func_id)))
        } else if let Some((async_ffi_function, signature)) = ctx.1.get(name) {
            let func_id = if !ctx.2.async_ffi_function_in_use.contains_key(name) {
                let func_id = ctx.2.async_ffi_function_in_use.len();
                ctx.2.async_ffi_function_in_use.insert(
                    name.to_string(),
                    (
                        async_ffi_function.clone(),
                        clone_signature(signature),
                        func_id
                    )
                );
                func_id
            } else {
                ctx.2.async_ffi_function_in_use.get(name).unwrap().2
            };
            Some(MantisGod::Right((true, func_id)))
        } else if let Some(func_id) = self.functions.get(name) {
            Some(MantisGod::Middle(*func_id))
        } else {
            self.lookup_with_parent(ctx, name)
        }
    }

    fn lookup_with_parent(&mut self, ctx: &mut LookupContext, name: &str) -> Option<LookupResult> {
        if let Some(parent) = self.parent.as_mut() {
            let result = parent.lookup(ctx, name);
            if let Some(inner) = result.as_ref() {
                if let MantisGod::Left(variable) = inner {
                    if let Some(function_frame) = self.function_frame.as_mut() {
                        let capture_id = function_frame.captures.len();
                        function_frame.captures.insert(
                            name.to_string(),
                            (capture_id, variable.0, variable.1)
                        );

                        self.variables.insert(name.to_string(), (true, capture_id));
                    }
                }
            }
            result
        } else {
            None
        }
    }

    fn add_var(&mut self, name: &str) -> usize {
        let var_id = self.allocate_reg();
        self.variables.insert(name.to_string(), (false, var_id));
        var_id
    }

    fn add_func(&mut self, name: &str) -> usize {
        let func_id = self.allocate_func();
        self.functions.insert(name.to_string(), func_id);
        func_id
    }

    fn allocate_reg(&mut self) -> usize {
        if let Some(function_frame) = self.function_frame.as_mut() {
            let reg = function_frame.register_alloc;
            function_frame.register_alloc += 1;
            reg
        } else {
            self.parent.as_mut()
                .expect("there must be a function frame")
                .allocate_reg()
        }
    }

    fn allocate_func(&mut self) -> usize {
        if let Some(parent) = self.parent.as_mut() {
            parent.allocate_func()
        } else {
            let func = self.function_counter;
            self.function_counter += 1;
            func
        }
    }
}
