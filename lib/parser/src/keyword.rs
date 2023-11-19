macro_rules! keywords {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(pub(crate) const $name: &'static str = $value;)*

        pub(crate) fn is_keyword(s: &str) -> bool {
            match s {
                $($name => true,)*
                _ => false,
            }
        }
    };
}

keywords!(
    IN => "in",
    ON => "on",
    NOT => "not",
    NULL => "null",
    TRUE => "true",
    FALSE => "false",
    INTO => "into",
    FROM => "from",
    JOIN => "join",
    WHERE => "where",
    LIMIT => "limit",
    INSERT => "insert",
    UPDATE => "update",
    DELETE => "delete",
    SELECT => "select",
    IMPORT => "import",
);
