use pr47::data::tyck::TyckInfoPool;
use pr47::data::Value;
use pr47::ffi::{DataOption, FFIException, Signature};
use pr47::ffi::sync_fn::{FunctionBase, value_into_ref_noalias, VMContext};
use xjbutil::boxed_slice;

pub struct DbgIntBind;

impl FunctionBase for DbgIntBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        Signature {
            func_type: tyck_info_pool.create_function_type(
                &[tyck_info_pool.get_int_type()],
                &[],
                &[]
            ),
            param_options: boxed_slice![DataOption::Raw],
            ret_option: boxed_slice![]
        }
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        _context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        eprintln!("DBG_INT = {}", args[0].vt_data.inner.int_value);
        *rets[0] = Value::new_null();
        Ok(())
    }

    unsafe fn call_unchecked<CTX: VMContext>(
        _context: &mut CTX,
        _args: &[Value],
        _rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        unimplemented!()
    }
}

pub struct DbgStringBind;

impl FunctionBase for DbgStringBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        Signature {
            func_type: tyck_info_pool.create_function_type(
                &[tyck_info_pool.get_string_type()],
                &[],
                &[]
            ),
            param_options: boxed_slice![DataOption::Raw],
            ret_option: boxed_slice![]
        }
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        _context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        let str = value_into_ref_noalias::<String>(args[0])?;
        eprintln!("DBG_STR = {}", str);
        *rets[0] = Value::new_null();
        Ok(())
    }

    unsafe fn call_unchecked<CTX: VMContext>(
        _context: &mut CTX,
        _args: &[Value],
        _rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        unimplemented!()
    }
}

pub const DBG_INT_BIND: DbgIntBind = DbgIntBind;
pub const DBG_STR_BIND: DbgStringBind = DbgStringBind;
