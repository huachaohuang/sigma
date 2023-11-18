use sigma_parser::Span;

#[derive(Debug)]
pub struct Error {
    pub span: Option<Span>,
    pub message: String,
}

impl Error {
    pub(crate) fn new(message: impl ToString) -> Self {
        Self {
            span: None,
            message: message.to_string(),
        }
    }

    pub(crate) fn with_span(span: Span, message: impl ToString) -> Self {
        Self {
            span: Some(span),
            message: message.to_string(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
