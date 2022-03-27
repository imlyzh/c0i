use pr47::data::tyck::TyckInfoPool;
use pr47::data::Value;
use pr47::ffi::{FFIException, Signature};
use pr47::ffi::sync_fn::{FunctionBase, VMContext};
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
            param_options: boxed_slice![],
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

pub const DBG_INT_BIND: DbgIntBind = DbgIntBind;
