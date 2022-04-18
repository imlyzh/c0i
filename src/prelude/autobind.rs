use std::collections::HashMap;
use std::convert::{Infallible, TryInto};

use sexpr_ir::gast::Handle;
use sexpr_ir::gast::symbol::{Location, Symbol};

use crate::value::Value;
use crate::value::callable::{Callable, NativeFunction, NativeInterface};
use crate::value::result::{CError, CResult};


pub trait RustCallable<TS, R, E> {
    fn call(&self, args: Vec<Value>) -> CResult;
}

impl<F, R> RustCallable<(), R, Infallible> for F
    where F: Fn() -> R,
          Value: From<R>
{
    fn call(&self, args: Vec<Value>) -> CResult {
        if args.len() != 0 {
            return Err(CError::ArgsNotMatching(0, args.len()));
        }

        let r: R = (self)();
        Ok(Value::from(r))
    }
}

macro_rules! impl_rust_callable {
    ($n:expr, $($tp:ident),*) => {
        #[allow(unused_parens)]
        impl<F, $($tp),*, R> RustCallable<($($tp),*), R, Infallible> for F
            where F: Fn($($tp),*) -> R,
                  $(Value: TryInto<$tp, Error = CError>,)*
                  Value: From<R>
        {
            #[allow(non_snake_case)]
            fn call(&self, args: Vec<Value>) -> CResult {
                if args.len() != $n {
                    return Err(CError::ArgsNotMatching($n, args.len()));
                }

                let mut iter = args.into_iter();
                $(
                    let $tp: $tp = iter.next().unwrap().try_into()?;
                )*
                let r: R = (self)($($tp),*);
                Ok(Value::from(r))
            }
        }
    }
}

impl_rust_callable!(1, T1);
impl_rust_callable!(2, T1, T2);
impl_rust_callable!(3, T1, T2, T3);
impl_rust_callable!(4, T1, T2, T3, T4);
impl_rust_callable!(5, T1, T2, T3, T4, T5);
impl_rust_callable!(6, T1, T2, T3, T4, T5, T6);
impl_rust_callable!(7, T1, T2, T3, T4, T5, T6, T7);
impl_rust_callable!(8, T1, T2, T3, T4, T5, T6, T7, T8);
impl_rust_callable!(9, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_rust_callable!(10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_rust_callable!(11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_rust_callable!(12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

impl<F, R, E> RustCallable<(), R, E> for F
    where F: Fn() -> Result<R, E>,
          Value: From<R>,
          CError: From<E>
{
    fn call(&self, args: Vec<Value>) -> CResult {
        if args.len() != 0 {
            return Err(CError::ArgsNotMatching(0, args.len()));
        }

        let r: Result<R, E> = (self)();
        match r {
            Ok(r) => Ok(Value::from(r)),
            Err(e) => Err(CError::from(e))
        }
    }
}

macro_rules! impl_rust_callable_exc {
    ($n:expr, $($tp:ident),*) => {
        #[allow(unused_parens)]
        impl<F, $($tp),*, R, E> RustCallable<($($tp),*), R, E> for F
            where F: Fn($($tp),*) -> Result<R, E>,
                  $(Value: TryInto<$tp, Error = CError>,)*
                  Value: From<R>,
                  CError: From<E>
        {
            #[allow(non_snake_case)]
            fn call(&self, args: Vec<Value>) -> CResult {
                if args.len() != $n {
                    return Err(CError::ArgsNotMatching($n, args.len()));
                }

                let mut iter = args.into_iter();
                $(
                    let $tp: $tp = match iter.next().unwrap().try_into() {
                        Ok(v) => v,
                        Err(e) => return Err(e)
                    };
                )*
                let r: Result<R, E> = (self)($($tp),*);
                match r {
                    Ok(r) => Ok(Value::from(r)),
                    Err(e) => Err(CError::from(e))
                }
            }
        }
    }
}

impl_rust_callable_exc!(1, T1);
impl_rust_callable_exc!(2, T1, T2);
impl_rust_callable_exc!(3, T1, T2, T3);
impl_rust_callable_exc!(4, T1, T2, T3, T4);
impl_rust_callable_exc!(5, T1, T2, T3, T4, T5);
impl_rust_callable_exc!(6, T1, T2, T3, T4, T5, T6);
impl_rust_callable_exc!(7, T1, T2, T3, T4, T5, T6, T7);
impl_rust_callable_exc!(8, T1, T2, T3, T4, T5, T6, T7, T8);
impl_rust_callable_exc!(9, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_rust_callable_exc!(10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_rust_callable_exc!(11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_rust_callable_exc!(12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

#[macro_export]
macro_rules! bind_rust_callable {
    ($fn_name:ident, $bound_fn_name:ident) => {
        fn $bound_fn_name(args: Vec<Value>) -> CResult {
            $crate::prelude::autobind::RustCallable::call(&$fn_name, args)
        }
    };
    ($vis:vis, $fn_name:ident, $bound_fn_name:ident) => {
        $vis fn $bound_fn_name(args: Vec<Value>) -> CResult {
            $crate::prelude::autobind::RustCallable::call(&$fn_name, args)
        }
    };
}

fn scope_register_rust_callable(
    locked_scope: &mut HashMap<Handle<Symbol>, Value>,
    location: Location,
    module_name: &'static str,
    registered_name: &'static str,
    bound_fn: NativeInterface
) {
    let symbol = Handle::new(
        Symbol(
            Handle::new(registered_name.to_string()),
            location.clone()
        )
    );

    if locked_scope.contains_key(&symbol) {
        panic!("Symbol `{}` already registered", registered_name);
    }

    locked_scope.insert(
        symbol,
        Value::Callable(
            Callable::Native(
                NativeFunction {
                    name: registered_name,
                    from_module: module_name,
                    is_pure: true,
                    interface: bound_fn
                }
            )
        )
    );
}

pub fn scope_register_module(
    locked_scope: &mut HashMap<Handle<Symbol>, Value>,
    module_name: &'static str,
    registered_functions: &[(&'static str, NativeInterface)]
) {
    let location = Location::new(Handle::new(String::from(module_name)), 0, 0, 0);
    for (registered_name, bound_fn) in registered_functions {
        scope_register_rust_callable(
            locked_scope,
            location.clone(),
            module_name,
            registered_name,
            *bound_fn
        );
    }
}
