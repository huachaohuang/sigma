use crate::Span;

#[derive(Clone, Debug)]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

impl Error {
    pub(crate) fn invalid_token(span: Span, message: impl ToString) -> Self {
        Self {
            span,
            kind: ErrorKind::InvalidToken(message.to_string()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    InvalidToken(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
