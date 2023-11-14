use crate::Span;

#[derive(Clone, Debug)]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub(crate) fn new(span: Span, kind: ErrorKind, message: impl ToString) -> Self {
        Self {
            span,
            kind,
            message: message.to_string(),
        }
    }

    pub(crate) fn incomplete(span: Span, message: impl ToString) -> Self {
        Self::new(span, ErrorKind::Incomplete, message)
    }

    pub(crate) fn invalid_token(span: Span, message: impl ToString) -> Self {
        Self::new(span, ErrorKind::InvalidToken, message)
    }

    pub(crate) fn unexpected_token(span: Span, message: impl ToString) -> Self {
        Self::new(span, ErrorKind::UnexpectedToken, message)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Incomplete,
    InvalidToken,
    UnexpectedToken,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
