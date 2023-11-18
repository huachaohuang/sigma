use crate::Span;

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
    pub message: String,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, span: Span, message: impl ToString) -> Self {
        Self {
            kind,
            span,
            message: message.to_string(),
        }
    }

    pub(crate) fn incomplete(span: Span, message: impl ToString) -> Self {
        Self::new(ErrorKind::Incomplete, span, message)
    }

    pub(crate) fn invalid_token(span: Span, message: impl ToString) -> Self {
        Self::new(ErrorKind::InvalidToken, span, message)
    }

    pub(crate) fn unexpected_token(span: Span, message: impl ToString) -> Self {
        Self::new(ErrorKind::UnexpectedToken, span, message)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Incomplete,
    InvalidToken,
    UnexpectedToken,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
