macro_rules! keywords {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(pub(crate) const $name: &'static str = $value;)*
    };
}

keywords!(
    IN => "in",
    NOT => "not",
    NULL => "null",
    TRUE => "true",
    FALSE => "false",
    IMPORT => "import",
);

pub(crate) fn is_keyword(s: &str) -> bool {
    match s {
        IN | NOT | NULL | TRUE | FALSE | IMPORT => true,
        _ => false,
    }
}
