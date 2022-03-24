use std::collections::HashMap;
use std::marker::PhantomData;

use crate::ast::TopLevel;
use crate::eval47::commons::{FFIAsyncFunction, FFIFunction, Signature};
use crate::eval47::data_map::{DataCollection, GValue};
use crate::eval47::util::MantisGod;

pub struct AnalyseContext {
    ffi_functions: HashMap<String, (FFIFunction, Signature)>,
    async_ffi_functions: HashMap<String, (FFIAsyncFunction, Signature)>,

    result: AnalyseResult,
    scope_chain: Option<Box<Scope>>
}

pub struct AnalyseResult {
    pub data_collection: DataCollection,
    pub global_data_map: HashMap<String, GValue>,
    pub ffi_function_in_use: HashMap<String, (FFIFunction, Signature, usize)>,
    pub async_ffi_function_in_use: HashMap<String, (FFIAsyncFunction, Signature, usize)>,
    _phantom: PhantomData<()>
}

impl AnalyseResult {
    fn new() -> AnalyseResult {
        AnalyseResult {
            data_collection: DataCollection::new(),
            global_data_map: HashMap::new(),
            ffi_function_in_use: HashMap::new(),
            async_ffi_function_in_use: HashMap::new(),
            _phantom: PhantomData
        }
    }
}

struct FunctionFrame {
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
    functions: HashMap<String, usize>
}

type LookupResult = MantisGod<
    (bool, usize), // variable
    usize, // script-defined function,
    (bool, usize) // FFI function
>;

type LookupContext<'a> = (
    &'a HashMap<String, (FFIFunction, Signature)>,
    &'a HashMap<String, (FFIAsyncFunction, Signature)>,
    &'a mut AnalyseResult
);

impl Scope {
    fn new(parent: Option<Box<Scope>>) -> Self {
        Self {
            parent,
            function_frame: None,
            variables: HashMap::new(),
            functions: HashMap::new()
        }
    }

    fn new_function_frame(parent: Option<Box<Scope>>) -> Self {
        Self {
            parent,
            function_frame: Some(FunctionFrame {
                captures: HashMap::new()
            }),
            variables: HashMap::new(),
            functions: HashMap::new()
        }
    }

    fn lookup(&mut self, ctx: &mut LookupContext, name: &str) -> Option<LookupResult> {
        if let Some(variable) = self.variables.get(name) {
            Some(MantisGod::Variable(variable.clone()))
        } else if let Some((ffi_function, signature)) = ctx.0.get(name) {
            if !ctx.2.ffi_function_in_use.contains_key(name) {
                ctx.2.ffi_function_in_use.insert(name.to_string(), (ffi_function.clone(), signature.clone(), 0));
            }
        } else {
            None
        }
    }
}