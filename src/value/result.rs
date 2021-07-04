use crate::value::Value;

pub type CResult = Result<Value, CError>;

#[derive(Debug, Clone)]
pub struct CError();
