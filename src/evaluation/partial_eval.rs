use sexpr_ir::gast::Handle;

use crate::{ast::{Call, Cond, Expr, Function, Let, TopLevel}, value::{Value, callable::{Callable, Closure}, scope::{Scope, SimpleScope}}};

use super::Eval;


pub trait PartialEval {
    type BackTraceType;
    fn partial_eval(&self, env: &Handle<Scope>) -> Result<Value, Self::BackTraceType>;
}

impl PartialEval for TopLevel {
    type BackTraceType = Self;
    fn partial_eval(&self, env: &Handle<Scope>) -> Result<Value, Self::BackTraceType> {
        match self {
            TopLevel::Function(f) => f.partial_eval(env)
            .map_err(|e| TopLevel::Function(Handle::new(e))),
            TopLevel::Bind(k, v) => v.partial_eval(env)
            .map_err(|e| TopLevel::Bind(k.clone(), e)),
            TopLevel::Expr(n) => n.partial_eval(env)
            .map_err(TopLevel::Expr),
        }
    }
}


impl PartialEval for Expr {
    type BackTraceType = Self;

    fn partial_eval(&self, env: &Handle<Scope>) -> Result<Value, Self::BackTraceType> {
        match self {
            Expr::Value(v) => Ok(v.clone()),
            Expr::Variable(k) => env.find(k).ok_or_else(|| self.clone()),
            Expr::Lambda(f) => f.partial_eval(env).map_err(|_| self.clone()),
            Expr::Let(l) => l.partial_eval(env).map_err(|e| Expr::Let(Handle::new(e))),
            Expr::Cond(c) => c.partial_eval(env).map_err(|e| Expr::Cond(Handle::new(e))),
            Expr::FunctionCall(f) => f.partial_eval(env)
                .map_err(|e| Expr::FunctionCall(Handle::new(e))),
        }
    }
}


impl PartialEval for Let {
    type BackTraceType = Self;

    fn partial_eval(&self, env: &Handle<Scope>) -> Result<Value, Self::BackTraceType> {
        todo!()
    }
}


impl PartialEval for Cond {
    type BackTraceType = Self;

    fn partial_eval(&self, env: &Handle<Scope>) -> Result<Value, Self::BackTraceType> {
        todo!()
    }
}


impl PartialEval for Call {
    type BackTraceType = Self;

    fn partial_eval(&self, env: &Handle<Scope>) -> Result<Value, Self::BackTraceType> {
        let mut backtract = false;
        let calls: Vec<_> = self.0
            .iter()
            .map(|e| match e.partial_eval(env) {
                Ok(v) => Expr::Value(v),
                Err(e) => {
                    backtract = true;
                    e
                },
            })
            .collect();
        todo!()
    }
}


impl PartialEval for Function {
    type BackTraceType = Self;

    fn partial_eval(&self, env: &Handle<Scope>) -> Result<Value, Self::BackTraceType> {
        self.eval(env)
        .map_err(|_|self.clone())
    }
}