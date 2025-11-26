use iced::widget::{Svg, svg};

pub fn refresh<'a>() -> Svg<'a> {
    const BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/refresh.svg"));
    svg(svg::Handle::from_memory(BYTES))
}
