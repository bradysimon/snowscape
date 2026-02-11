use iced::widget::{Svg, svg};

pub fn checkmark<'a>() -> Svg<'a> {
    const BYTES: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/checkmark.svg"));
    svg(svg::Handle::from_memory(BYTES))
}

pub fn circle_slash<'a>() -> Svg<'a> {
    const BYTES: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/circle-slash.svg"
    ));
    svg(svg::Handle::from_memory(BYTES))
}

pub fn refresh<'a>() -> Svg<'a> {
    const BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/refresh.svg"));
    svg(svg::Handle::from_memory(BYTES))
}

pub fn undo<'a>() -> Svg<'a> {
    const BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/undo.svg"));
    svg(svg::Handle::from_memory(BYTES))
}

pub fn xmark<'a>() -> Svg<'a> {
    const BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/xmark.svg"));
    svg(svg::Handle::from_memory(BYTES))
}
