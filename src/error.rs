#[derive(Debug, Clone)]
pub enum CompilerError {
    // SexprIrParseError,
    BadSyntax(String),
    IsNotSymbol(String),
    IncompleteExpr(String),
    InvalidPairRight(String),
    InvalidExprLength(usize, usize, String),
    InvalidExprType((), String)
}

pub(crate) fn bad_syntax<T: ToString>(i: &T) -> CompilerError {
    CompilerError::BadSyntax(i.to_string())
}

pub(crate) fn invalid_list_tail<T: ToString>(i: &T) -> CompilerError {
    CompilerError::InvalidPairRight(i.to_string())
}

pub(crate) fn incomplete_expr<T: ToString>(i: &T) -> CompilerError {
    CompilerError::IncompleteExpr(i.to_string())
}

pub(crate) fn invalid_expr_length<T: ToString>(i: &T, takes: usize, give: usize) -> CompilerError {
    CompilerError::InvalidExprLength(takes, give, i.to_string())
}

pub(crate) fn invalid_expr_type<T: ToString>(i: &T, etype: ()) -> CompilerError {
    CompilerError::InvalidExprType(etype, i.to_string())
}