use std::any::TypeId;
use std::io;
use pr47::builtins::closure::Closure;
use pr47::builtins::object::Object;
use pr47::builtins::vec::VMGenericVec;
use pr47::data::generic::GenericTypeVT;
use pr47::data::tyck::TyckInfoPool;
use pr47::data::Value;
use pr47::data::value_typed::{VALUE_TYPE_TAG_MASK, ValueTypeTag};
use pr47::ffi::{DataOption, FFIException, Signature};
use pr47::ffi::sync_fn::{FunctionBase, value_into_ref_noalias, VMContext};
use xjbutil::boxed_slice;
use xjbutil::unchecked::{UncheckedCellOps, UnsafeFrom};

macro_rules! implement_blanket_call_unchecked {
    () => {
        unsafe fn call_unchecked<CTX: VMContext>(
            _context: &mut CTX,
            _args: &[Value],
            _rets: &[*mut Value]
        ) -> Result<(), FFIException> {
            unimplemented!("`call_unchecked` should never be used for eval47")
        }
    }
}

pub struct DisplayBind;
pub const DISPLAY_BIND: DisplayBind = DisplayBind;

unsafe fn display_value(value: Value) {
    if value.is_value() {
        match ValueTypeTag::unsafe_from((value.vt_data.tag as u8) & VALUE_TYPE_TAG_MASK) {
            ValueTypeTag::Int => eprint!("{}", value.vt_data.inner.int_value),
            ValueTypeTag::Float => eprint!("{}", value.vt_data.inner.float_value),
            ValueTypeTag::Char => eprint!("{}", value.vt_data.inner.char_value),
            ValueTypeTag::Bool => eprint!("{}", value.vt_data.inner.bool_value)
        }
    } else if value.is_container() {
        let vt = value.ptr_repr.trivia as *const GenericTypeVT;
        if (&*vt).tyck_info.as_ref().type_id == TypeId::of::<Closure>() {
            if value.ownership_info().is_readable() {
                let closure = &*(value.get_as_mut_ptr::<Closure>() as *const Closure);
                eprint!("(closure:{} captures=#(", closure.func_id);
                for (idx, capture) in closure.captures.iter().enumerate() {
                    display_value(*capture);
                    if idx != closure.captures.len() - 1 {
                        eprint!(", ");
                    }
                }
                eprint!("))");
            } else {
                eprint!("(unreadable closure)")
            }
        } else if (&*vt).tyck_info.as_ref().type_id == TypeId::of::<VMGenericVec>() {
            if value.ownership_info().is_readable() {
                let vec = &*(value.get_as_mut_ptr::<VMGenericVec>() as *const VMGenericVec);
                eprint!("#(");
                let inner = vec.inner.get_ref_unchecked();
                for (idx, element) in inner.iter().enumerate() {
                    display_value(*element);
                    if idx != inner.len() - 1 {
                        eprint!(", ");
                    }
                }
                eprint!(")");
            } else {
                eprint!("(unreadable vector)")
            }
        }
    } else {
        if value.ownership_info_norm().is_readable() {
            let dyn_base = value.get_as_dyn_base();
            if (&*dyn_base).dyn_type_id() == TypeId::of::<String>() {
                eprint!("{}", value_into_ref_noalias::<String>(value).unwrap());
            } else if (&*dyn_base).dyn_type_id() == TypeId::of::<Object>() {
                eprint!("(object Object)");
            } else {
                eprint!("(non-displayable)");
            }
        } else {
            eprint!("(unreadable value)")
        }
    }
}

impl FunctionBase for DisplayBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        Signature {
            func_type: tyck_info_pool.create_function_type(
                &[tyck_info_pool.get_any_type()],
                &[],
                &[]
            ),
            param_options: boxed_slice![DataOption::RawUntyped],
            ret_option: boxed_slice![]
        }
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        _context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        for arg in args {
            display_value(*arg)
        }

        *rets[0] = Value::new_bool(false);
        Ok(())
    }

    implement_blanket_call_unchecked!{}
}

pub struct ReadLineBind;
pub const READ_LINE_BIND: ReadLineBind = ReadLineBind;

impl FunctionBase for ReadLineBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        Signature {
            func_type: tyck_info_pool.create_function_type(
                &[],
                &[],
                &[tyck_info_pool.get_string_type()]
            ),
            param_options: boxed_slice![],
            ret_option: boxed_slice![DataOption::Move]
        }
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        context: &mut CTX,
        _args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        let mut s = String::new();
        match io::stdin().read_line(&mut s) {
            Ok(_) => {
                s = s.trim().to_string();
                let value = Value::new_owned(s);
                context.add_heap_managed(value);
                *rets[0] = value;
                Ok(())
            },
            Err(e) => {
                let e = Value::new_owned(e);
                context.add_heap_managed(e);
                Err(FFIException::Checked(e))
            }
        }
    }

    implement_blanket_call_unchecked!{}
}


pub struct ParseIntBind;
pub const PARSE_INT_BIND: ParseIntBind = ParseIntBind;

impl FunctionBase for ParseIntBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        Signature {
            func_type: tyck_info_pool.create_function_type(
                &[tyck_info_pool.get_string_type()],
                &[],
                &[tyck_info_pool.get_int_type()]
            ),
            param_options: boxed_slice![DataOption::Share],
            ret_option: boxed_slice![DataOption::Move]
        }
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        let s: &String = value_into_ref_noalias(args[0])?;
        match s.parse::<i64>() {
            Ok(i) => {
                let value = Value::new_int(i);
                *rets[0] = value;
                Ok(())
            },
            Err(e) => {
                let e = Value::new_owned(e);
                context.add_heap_managed(e);
                Err(FFIException::Checked(e))
            }
        }
    }

    implement_blanket_call_unchecked!{}
}
