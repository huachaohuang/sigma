use sigma_parser::Span;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub span: Option<Span>,
}

impl Error {
    pub(crate) fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
            span: None,
        }
    }

    pub(crate) fn with_span(message: impl ToString, span: Span) -> Self {
        Self {
            message: message.to_string(),
            span: Some(span),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
