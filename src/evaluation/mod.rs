pub mod debug_eval;
pub mod call;
// pub mod partial_eval;
// pub mod partial_call;


use std::process::exit;

use sexpr_ir::gast::Handle;
use sexpr_ir::syntax::sexpr::file_parse;

use crate::sexpr_to_ast::FromSexpr;
use crate::value::Value;
use crate::value::callable::{Callable, Closure};
use crate::value::result::CError;
use crate::value::scope::SimpleScope;
use crate::value::{result::CResult, scope::Scope};

use crate::ast::*;
use call::Call;


pub trait Eval {
    fn eval(&self, env: &Handle<Scope>) -> CResult;
}

impl Eval for ModuleTop {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        match self {
            ModuleTop::TopLevel(t) => t.eval(env),
            ModuleTop::Import(x) => {
                load_file(todo!(), env);
                Ok(Value::Nil)
            },
        }
    }
}


pub fn load_file(path: &str, env: &Handle<Scope>) -> Result<(), CError> {
    let file = file_parse(path).unwrap();
    let file: Result<Vec<_>, _> = file
        .iter()
        .map(TopLevel::from_sexpr)
        .collect();
    if let Err(x) = file.clone() {
        println!("Complie Error: {:?}", x);
        exit(-1);
    }
    let file = file.unwrap();
    file
        .iter()
        .try_for_each(|x| x.eval(&env).map(|_| ()))?;
    Ok(())
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
                .ok_or_else(|| CError::SymbolNotFound(k.clone())),
            Expr::Value(x) => Ok(x.clone()),
            Expr::Let(x) => x.eval(env),
            Expr::Cond(x) => x.eval(env),
            Expr::Lambda(x) => x.eval(env),
            Expr::FunctionCall(x) => x.eval(env),
            Expr::Set(x) => x.eval(env),
        }
    }
}

impl Eval for Set {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        env.set(&self.name, &self.value.eval(env)?);
        Ok(Value::Nil)
    }
}

impl Eval for Let {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        let this_level = SimpleScope::new();
        {
            let mut record = this_level.0.write().unwrap();
            for (k, v) in &self.binds {
                let v = v.eval(env)
                .map_err(|x| CError::Positional(
                    self.pos.clone(),
                    Handle::new(x)))?;
                record.insert(k.clone(), v);
            }
        }
        let env = env.new_level(this_level);
        if self.bodys.is_empty() {
            Ok(Value::Nil)
        } else if self.bodys.len() == 1 {
            self.bodys.first().unwrap().eval(&env)
            .map_err(|x| CError::Positional(
                self.pos.clone(),
                Handle::new(x)))
        } else {
            let body_end = self.bodys.last().unwrap();
            let bodys = &self.bodys[..self.bodys.len()-1];
            for i in bodys {
                i.eval(&env)
                .map_err(|x| CError::Positional(
                    self.pos.clone(),
                    Handle::new(x)))?;
            }
            body_end.eval(&env)
            .map_err(|x| CError::Positional(
                self.pos.clone(),
                Handle::new(x)))
        }
    }
}

impl Eval for Cond {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        for (cond, expr) in &self.pairs {
            let c = cond
            .eval(env)
            .map_err(|x| CError::Positional(
                self.pos.clone(),
                Handle::new(x)))?;
            if let Value::Bool(c) = c {
                if c {
                    return expr.eval(env)
                    .map_err(|x| CError::Positional(
                        self.pos.clone(),
                        Handle::new(x)));
                }
            } else {
                // return error "cond is not boolean"
                return Err(CError::Positional(
                    self.pos.clone(),
                    Handle::new(CError::CondIsNotBoolean(c))));
            }
        }
        if let Some(x) = &self.other {
            x.eval(env)
        } else {
            // return error "conds is not matching"
            Err(CError::Positional(
                self.pos.clone(),
                Handle::new(CError::CondIsNotMatching)))
        }
    }
}

impl Eval for crate::ast::Call {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        let r: Result<Vec<_>, _> = self.0.iter().map(|x| x.eval(env)).collect();
        let r = r?;
        debug_assert_ne!(r.len(), 0);
        let mut iter = r.into_iter();
        let value = iter.next().unwrap();
        if let Value::Callable(x) = value {
            let args: Vec<_> = iter.collect();
            Call::call(&x, &args)
            .map_err(|e| CError::StackBacktrace(x, Handle::new(e)))
        } else {
            Err(CError::ValueIsNotCallable(value))
        }
    }
}

impl Eval for Function {
    fn eval(&self, env: &Handle<Scope>) -> CResult {
        let env = env.new_level(SimpleScope::new());
        let r = Closure(self.clone(), Some(env));
        Ok(Value::Callable(Callable::Closure(Handle::new(r))))
    }
}
