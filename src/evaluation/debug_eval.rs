use crate::{ast::TopLevel, value::result::CError};




trait DebugEval {
    type Target;
    fn debug_eval(root: TopLevel, pos: ()) -> Result<Self::Target, CError>;
}