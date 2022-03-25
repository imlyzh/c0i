use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::ast::*;



pub trait FreeVariables {
    fn free_variables(&self, env: &mut Vec<Handle<Symbol>>) -> Vec<Handle<Symbol>>;
}


impl FreeVariables for TopLevel {
    fn free_variables(&self, env: &mut Vec<Handle<Symbol>>) -> Vec<Handle<Symbol>> {
        match self {
            TopLevel::Function(v) => {
                env.push(v.name.clone().unwrap());
                v.free_variables(env)
            },
            TopLevel::Bind(n, v) => {
                env.push(n.clone());
                v.free_variables(env)
            },
            TopLevel::Expr(v) => v.free_variables(env),
        }
    }
}


impl FreeVariables for Expr {
    fn free_variables(&self, env: &mut Vec<Handle<Symbol>>) -> Vec<Handle<Symbol>> {
        match self {
            Expr::Variable(v) => if env.contains(v) {
                vec![]
            } else {
                vec![v.clone()]
            },
            Expr::Lambda(v) => v.free_variables(env),
            Expr::Let(v) => v.free_variables(env),
            Expr::Cond(v) => v.free_variables(env),
            Expr::FunctionCall(v) => v.free_variables(env),
            Expr::Value(_) => vec![],
            Expr::Set(_) => todo!(),
        }
    }
}


impl FreeVariables for Let {
    fn free_variables(&self, env: &mut Vec<Handle<Symbol>>) -> Vec<Handle<Symbol>> {
        let mut fv_record: Vec<Handle<Symbol>> = self.binds
        .iter()
        .flat_map(|(_, v)| v.free_variables(env))
        .collect();

        let mut names: Vec<Handle<Symbol>> = self.binds
        .iter()
        .map(|(n, _)| n.clone())
        .collect();

        let mut env = env.clone();
        env.append(&mut names);

        let mut body_fv: Vec<Handle<Symbol>> = self.body
        .iter()
        .flat_map(|x| x.free_variables(&mut env))
        .collect();

        fv_record.append(&mut body_fv);

        fv_record
    }
}


impl FreeVariables for Cond {
    fn free_variables(&self, env: &mut Vec<Handle<Symbol>>) -> Vec<Handle<Symbol>> {
        let mut r: Vec<Handle<Symbol>> = self.pairs.iter()
        .flat_map(|(a, b)| vec![a, b])
        .flat_map(|a| a.free_variables(env))
        .collect();
        if let Some(other) = self.other.clone() {
            r.append(&mut other.free_variables(env));
        }
        r
    }
}


impl FreeVariables for Call {
    fn free_variables(&self, env: &mut Vec<Handle<Symbol>>) -> Vec<Handle<Symbol>> {
        self.0.iter().flat_map(|x| x.free_variables(env)).collect()
    }
}


impl FreeVariables for Function {
    fn free_variables(&self, env: &mut Vec<Handle<Symbol>>) -> Vec<Handle<Symbol>> {
        let mut env = env.clone();
        let mut prarms = self.params.clone();
        env.append(&mut prarms);
        if let Some(x) = self.extend_params.clone() {
            env.push(x);
        }

        let fv_record: Vec<Handle<Symbol>> = self.body
        .iter()
        .flat_map(|x| x.free_variables(&mut env))
        .collect();

        fv_record
    }
}
