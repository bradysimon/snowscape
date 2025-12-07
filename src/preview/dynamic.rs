mod extract_params;
pub mod param;
pub mod stateful;
pub mod stateless;

use std::ops::RangeInclusive;

pub use extract_params::ExtractParams;
use iced::Color;
pub use param::{Param, boolean, color, number, select, slider, text};
pub use stateful::stateful;
pub use stateless::stateless;

/// A dynamic parameter value used within [`Param`].
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A boolean toggle.
    Bool(bool),
    /// A customizable text value.
    Text(String),
    /// A 32-bit integer value.
    I32(i32),
    /// A selection from a list of options. Stores (selected_index, options).
    Select(usize, Vec<String>),
    /// A slider value with range. Stores (current, range).
    Slider(f32, RangeInclusive<f32>),
    /// A color value.
    Color(Color),
}
