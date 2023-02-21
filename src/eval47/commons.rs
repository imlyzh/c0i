use pr47::ffi::async_fn::AsyncFunction;
use pr47::ffi::sync_fn::Function;
use pr47::vm::al31f::compiled::{
    CompiledFunction as CF,
    CompiledProgram as CP
};
use pr47::vm::al31f::{AL31F, AsyncCombustor, Combustor};
use pr47::vm::al31f::alloc::default_alloc::DefaultAlloc;

pub use pr47::ffi::{DataOption, FFIException, Signature};
pub use pr47::ffi::async_fn::{AsyncFunctionBase, AsyncVMContext};
pub use pr47::ffi::sync_fn::{FunctionBase, VMContext};

pub type Alloc = DefaultAlloc;
pub type VM = AL31F<Alloc>;
pub type CompiledFunction = CF;
pub type CompiledProgram = CP<Alloc>;
pub type FFIFunction = &'static dyn Function<Combustor<Alloc>>;
pub type FFIAsyncFunction = &'static dyn AsyncFunction<VM, AsyncCombustor<Alloc>>;
