#[derive(Debug, Clone)]
pub enum CompilerError {
    // SexprIrParseError,
    BadSyntax(String),
    IsNotSymbol(String),
    IncompleteExpr(String),
}

pub(crate) fn bad_syntax<T: ToString>(i: &T) -> CompilerError {
    CompilerError::BadSyntax(i.to_string())
}
