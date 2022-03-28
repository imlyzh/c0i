use crate::value::{Value, callable::{Callable, Closure, NativeFunction}, result::CResult, scope::Scope};

use super::Eval;



pub trait Call {
    fn call(&self, args: &[Value]) -> CResult;
}

impl Call for Callable {
    fn call(&self, args: &[Value]) -> CResult {
        match self {
            Callable::Closure(x) => x.call(args),
            Callable::Native(x) => x.call(args),
        }
    }
}


impl Call for Closure {
    fn call(&self, args: &[Value]) -> CResult {
        let Closure(f, env) = self;
        let args_dict = f.match_args(&args)?;

        let scope = if let Some(env) = env {
            env.new_level(args_dict)
        } else {
            Scope::from(args_dict)
        };

        if f.bodys.is_empty() {
            Ok(Value::Nil)
        } else if f.bodys.len() == 1 {
            f.bodys.first().unwrap().eval(&scope)
        } else {
            let body_end = f.bodys.last().unwrap();
            let bodys = &f.bodys[..f.bodys.len()-1];
            for i in bodys {
                i.eval(&scope)?;
            }
            body_end.eval(&scope)
        }
    }
}

impl Call for NativeFunction {
    fn call(&self, i: &[Value]) -> CResult {
        (self.interface)(i.to_vec())
    }
}
