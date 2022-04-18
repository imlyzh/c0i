extern crate core;

pub mod value;
pub mod ast;

#[cfg(feature = "c0i")] pub mod evaluation;
#[cfg(feature = "c0i")] pub mod sexpr_to_ast;
#[cfg(feature = "c0i")] pub mod analysis;
#[cfg(feature = "c0i")] pub mod error;
#[cfg(feature = "c0i")] pub mod prelude;

#[cfg(feature = "c047")]
pub mod eval47;
