pub mod error;
pub mod to_literal;
pub mod raw_operator;
pub mod native_eq_ord_operator;
pub mod dynamic_type_check;
pub mod native_math_operator;
pub mod native_bool_operator;
pub mod native_string_operator;
pub mod native_dict_operator;
pub mod io_operator;

use sexpr_ir::gast::Handle;

use error::*;
use to_literal::*;
use raw_operator::*;
use native_eq_ord_operator::*;
use dynamic_type_check::*;
use native_math_operator::*;
use native_string_operator::*;
use native_bool_operator::*;
use native_dict_operator::*;
use io_operator::*;

use crate::value::autobind::scope_register_module;
use crate::value::scope::Scope;

pub fn init() -> Handle<Scope> {
    let record = Scope::new();

    {
        let mut rcd = record.this_level.0.write().unwrap();

        scope_register_module(&mut rcd, "<builtin>", &[
            ("error", native_error),
            ("unreachable", native_unreachable),
            ("literal", literal),
            ("read", read),
            ("car", car),
            ("cdr", cdr),
            ("cons", cons),
            ("make-vector", vector),
            ("vec-reduce", vector_reduce),
            ("set-vec!", set_vector),
            ("id", id),
            ("ignore", ignore),
            ("eq?", eq),
            ("ne?", ne),
            ("lt?", lt),
            ("gt?", gt),
            ("le?", le),
            ("ge?", ge),
            ("nil?", native_is_nil),
            ("char?", native_is_char),
            ("bool?", native_is_bool),
            ("int?", native_is_int),
            ("uint?", native_is_uint),
            ("float?", native_is_float),
            ("str?", native_is_str),
            ("sym?", native_is_sym),
            ("pair?", native_is_pair),
            ("dict?", native_is_dict),
            ("vec?", native_is_vec),
            ("callable?", native_is_callable),
            ("+i", native_add_int),
            ("+u", native_add_uint),
            ("+f", native_add_float),
            ("-i", native_sub_int),
            ("-u", native_sub_uint),
            ("-f", native_sub_float),
            ("*i", native_mul_int),
            ("*u", native_mul_uint),
            ("*f", native_mul_float),
            ("/i", native_div_int),
            ("/u", native_div_uint),
            ("/f", native_div_float),
            ("+s", native_add_str),
            ("->string", to_str),
            ("not", native_bool_not),
            ("and", native_bool_and),
            ("or", native_bool_or),
            ("make-dict", make_dict),
            ("read-stdin", read_stdin),
            ("read-line", read_line),
            ("display", display),
            ("displayln", displayln),
            ("file->string", file_to_string),
            ("write-file", write_file)
        ])
    }
    record
}
