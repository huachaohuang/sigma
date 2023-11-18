use sigma_parser::Span;

#[derive(Debug)]
pub struct Error {
    pub span: Span,
    pub message: String,
}

impl Error {
    pub(crate) fn new(message: impl ToString) -> Self {
        Self::with_span(Span::default(), message)
    }

    pub(crate) fn with_span(span: Span, message: impl ToString) -> Self {
        Self {
            span,
            message: message.to_string(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
