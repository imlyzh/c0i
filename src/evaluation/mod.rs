use std::collections::HashMap;

use sexpr_ir::gast::Handle;
use sexpr_ir::gast::symbol::Symbol;

use crate::analysis::free_variable::FreeVariables;
use crate::value::Value;
use crate::value::callable::{Call, Callable, Closure};
use crate::value::result::CError;
use crate::value::scope::SimpleScope;
use crate::value::{result::CResult, scope::Scope};

use crate::ast::*;


pub trait Eval {
    fn eval(&self, env: &Handle<Scope>) -> CResult;
}


impl Eval for TopLevel {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        match self {
            TopLevel::Function(x) => {
                let v = x.eval(env)?;
                let mut record = env.this_level.0.write().unwrap();
                record.insert(x.name.clone().unwrap(), v);
                Ok(Value::Nil)
            },
            TopLevel::Bind(k, v) => {
                let v = v.eval(env)?;
                env.this_level.0.write().unwrap().insert(k.clone(), v);
                Ok(Value::Nil)
            },
            TopLevel::Expr(v) => v.eval(env),
        }
    }
}


impl Eval for Expr {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        match self {
            Expr::Variable(k) => env
                .find(k)
                .ok_or_else(|| CError()),
            Expr::Value(x) => Ok(x.clone()),
            Expr::Let(x) => x.eval(env),
            Expr::Cond(x) => x.eval(env),
            Expr::Lambda(x) => x.eval(env),
            Expr::FunctionCall(x) => x.eval(env),
        }
    }
}


impl Eval for Let {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        let this_level = SimpleScope::new();
        {
            let mut record = this_level.0.write().unwrap();
            for (k, v) in &self.binds {
                let v = v.eval(env)?;
                record.insert(k.clone(), v);
            }
        }
        let env = env.new_level(this_level);
        if self.bodys.is_empty() {
            Ok(Value::Nil)
        } else if self.bodys.len() == 1 {
            self.bodys.first().unwrap().eval(&env)
        } else {
            let body_end = self.bodys.last().unwrap();
            let bodys = &self.bodys[..self.bodys.len()-1];
            for i in bodys {
                i.eval(&env)?;
            }
            body_end.eval(&env)
        }
    }
}

impl Eval for Cond {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        for (cond, expr) in &self.pairs {
            let c = cond.eval(env)?;
            if let Value::Bool(c) = c {
                if c {
                    return expr.eval(env);
                }
            } else {
                // return error "cond is not boolean"
                return Err(CError());
            }
        }
        if let Some(x) = &self.other {
            x.eval(env)
        } else {
            // return error "conds is not matching"
            Err(CError())
        }
    }
}

impl Eval for crate::ast::Call {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        let r: Result<Vec<_>, _> = self.0.iter().map(|x| x.eval(env)).collect();
        let r = r?;
        debug_assert_ne!(r.len(), 0);
        let mut iter = r.into_iter();
        if let Value::Callable(x) = iter.next().unwrap() {
            x.call(iter.collect())
        } else {
            Err(CError())
        }
    }
}

impl Eval for Function {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        let mut variable_env = vec![];
        let capture  = self.free_variables(&mut variable_env);
        
        let env: Option<HashMap<Handle<Symbol>, Value>> = capture.iter().map(|k| env
            .find(k)
            .map(|v| (k.clone(), v)))
            .collect();
        let env = env.ok_or_else(|| CError())?;
        let env = Scope::from(SimpleScope::from(env));
        let r = Closure(self.clone(), Some(env));
        Ok(Value::Callable(Callable::Closure(Handle::new(r))))
    }
}
