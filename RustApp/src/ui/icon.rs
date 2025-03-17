#[macro_export]
macro_rules! widget_icon_handle {
    ($name:literal) => {{
        let bytes = &include_bytes!(concat!("../../res/icons/", $name, ".svg"))[..];
        cosmic::widget::icon::from_svg_bytes(bytes).symbolic(true)
    }};
}

#[macro_export]
macro_rules! widget_icon {
    ($name:literal) => {{
        use $crate::widget_icon_handle;

        cosmic::widget::icon::icon(widget_icon_handle!($name))
    }};
}
#[macro_export]
macro_rules! widget_icon_button {
    ($name:literal) => {{
        use $crate::widget_icon_handle;

        cosmic::widget::button::icon(widget_icon_handle!($name))
    }};
}

#[macro_export]
macro_rules! window_icon {
    ($name:literal, $width:expr, $height:expr) => {{
        let svg = include_bytes!(concat!("../../res/icons/", $name, ".svg"));
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(svg, &opt).unwrap();
        let viewbox = tree.size();

        let mut pixmap = resvg::tiny_skia::Pixmap::new($width, $height).unwrap();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(
                $width as f32 / viewbox.width(),
                $height as f32 / viewbox.height(),
            ),
            &mut pixmap.as_mut(),
        );

        let rgba = pixmap.data().to_vec();

        cosmic::iced::window::icon::from_rgba(rgba, $width, $height).ok()
    }};
    ($name:literal) => {{
        window_icon!($name, 32, 32)
    }};
}
