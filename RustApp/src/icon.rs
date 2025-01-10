#[macro_export]
macro_rules! icon_handle {
    ($name:literal) => {{
        let bytes = include_bytes!(concat!("../res/icons/", $name, ".svg"));
        cosmic::widget::icon::from_svg_bytes(bytes).symbolic(true)
    }};
}

#[macro_export]
macro_rules! icon {
    ($name:literal) => {{
        use $crate::icon_handle;

        cosmic::widget::icon::icon(icon_handle!($name))
    }};
}
#[macro_export]
macro_rules! icon_button {
    ($name:literal) => {{
        use $crate::icon_handle;
        cosmic::widget::button::icon(icon_handle!($name))
    }};
}
