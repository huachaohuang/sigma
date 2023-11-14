macro_rules! keywords {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(pub(crate) const $name: &'static str = $value;)*
    };
}

keywords!(
    IN => "in",
    NOT => "not",
);
