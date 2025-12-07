use std::{fmt::Display, ops::RangeInclusive};

use iced::Color;

use crate::dynamic::Value;

/// A dynamic parameter that can be adjusted in the configuration pane.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// The display name of the parameter.
    pub name: String,
    /// The current value of the parameter.
    pub value: Value,
}

impl Param {
    /// Create a new dynamic parameter with the given `name` and `value`.
    pub fn new(name: impl Into<String>, value: impl Into<Value>) -> Self {
        Param {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::I32(value)
    }
}

/// A trait for dynamic parameters that can be adjusted and provide typed values.
pub trait DynamicParam: Clone + Send + 'static {
    /// The type of value this parameter produces.
    /// This is the type that will be available in the closure used to build the preview.
    type Value: Clone + Send + 'static;

    /// Get the name of this parameter.
    fn name(&self) -> &str;

    /// Get the current value as a Param for UI display.
    fn to_param(&self) -> Param;

    /// Update this parameter given a new `value`.
    /// Typically, this only updates the internal state if the `value` is the same variant.
    fn update(&mut self, value: Value);

    /// Gets the typed value.
    fn value(&self) -> Self::Value;
}

/// A text parameter that produces String values.
#[derive(Debug, Clone)]
pub struct TextParam {
    name: String,
    value: String,
}

impl DynamicParam for TextParam {
    type Value = String;

    fn name(&self) -> &str {
        &self.name
    }

    fn to_param(&self) -> Param {
        Param::new(&self.name, Value::Text(self.value.clone()))
    }

    fn update(&mut self, value: Value) {
        if let Value::Text(text) = value {
            self.value = text;
        }
    }

    fn value(&self) -> Self::Value {
        self.value.clone()
    }
}

/// A number parameter that produces i32 values.
#[derive(Debug, Clone)]
pub struct NumberParam {
    name: String,
    value: i32,
}

impl DynamicParam for NumberParam {
    type Value = i32;

    fn name(&self) -> &str {
        &self.name
    }

    fn to_param(&self) -> Param {
        Param::new(&self.name, Value::I32(self.value))
    }

    fn update(&mut self, value: Value) {
        if let Value::I32(num) = value {
            self.value = num;
        }
    }

    fn value(&self) -> Self::Value {
        self.value
    }
}

/// A boolean parameter that produces bool values.
#[derive(Debug, Clone)]
pub struct BoolParam {
    name: String,
    value: bool,
}

impl DynamicParam for BoolParam {
    type Value = bool;

    fn name(&self) -> &str {
        &self.name
    }

    fn to_param(&self) -> Param {
        Param::new(&self.name, Value::Bool(self.value))
    }

    fn update(&mut self, value: Value) {
        if let Value::Bool(b) = value {
            self.value = b;
        }
    }

    fn value(&self) -> Self::Value {
        self.value
    }
}

/// Create a dynamic text parameter.
///
/// # Example
///
/// ```
/// use snowscape::dynamic;
/// let title = dynamic::text("Title", "Hello, World!");
/// ```
pub fn text(name: impl Into<String>, value: impl Into<String>) -> TextParam {
    TextParam {
        name: name.into(),
        value: value.into(),
    }
}

/// Create a dynamic number parameter.
///
/// # Example
///
/// ```
/// use snowscape::dynamic;
/// let font_size = dynamic::number("Font Size", 24);
/// ```
pub fn number(name: impl Into<String>, value: i32) -> NumberParam {
    NumberParam {
        name: name.into(),
        value,
    }
}

/// Create a dynamic boolean parameter.
///
/// # Example
///
/// ```
/// use snowscape::dynamic;
/// let is_enabled = dynamic::boolean("Enabled", true);
/// ```
pub fn boolean(name: impl Into<String>, value: bool) -> BoolParam {
    BoolParam {
        name: name.into(),
        value,
    }
}

/// A select parameter that allows choosing from a list of typed options.
///
/// The type `T` must implement `Display` for rendering in the UI,
/// and `Clone + PartialEq + Send + 'static` for type-safe value handling.
#[derive(Debug, Clone)]
pub struct SelectParam<T> {
    name: String,
    options: Vec<T>,
    selected_index: usize,
}

impl<T> DynamicParam for SelectParam<T>
where
    T: std::fmt::Display + Clone + PartialEq + Send + 'static,
{
    type Value = T;

    fn name(&self) -> &str {
        &self.name
    }

    fn to_param(&self) -> Param {
        let option_strings: Vec<String> = self.options.iter().map(|o| o.to_string()).collect();
        Param::new(
            &self.name,
            Value::Select(self.selected_index, option_strings),
        )
    }

    fn update(&mut self, value: Value) {
        if let Value::Select(index, _) = value {
            if index < self.options.len() {
                self.selected_index = index;
            }
        }
    }

    fn value(&self) -> Self::Value {
        self.options[self.selected_index].clone()
    }
}

/// Create a dynamic select parameter that allows choosing from a list of options.
///
/// The `options` slice must contain at least one element, and `default` must be
/// present in the options list.
///
/// # Example
///
/// ```
/// use snowscape::dynamic;
/// #[derive(Debug, Clone, PartialEq)]
/// enum Alignment { Left, Center, Right }
///
/// impl std::fmt::Display for Alignment {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             Alignment::Left => write!(f, "Left"),
///             Alignment::Center => write!(f, "Center"),
///             Alignment::Right => write!(f, "Right"),
///         }
///     }
/// }
///
/// let alignment = dynamic::select(
///     "Alignment",
///     &[Alignment::Left, Alignment::Center, Alignment::Right],
///     Alignment::Center
/// );
/// ```
pub fn select<T>(name: impl Into<String>, options: &[T], default: T) -> SelectParam<T>
where
    T: Display + Clone + PartialEq,
{
    let selected_index = options
        .iter()
        .position(|o| *o == default)
        .expect("default value must be present in options");

    SelectParam {
        name: name.into(),
        options: options.to_vec(),
        selected_index,
    }
}

/// A slider parameter that produces f32 values within a range.
#[derive(Debug, Clone)]
pub struct SliderParam {
    name: String,
    value: f32,
    range: RangeInclusive<f32>,
}

impl DynamicParam for SliderParam {
    type Value = f32;

    fn name(&self) -> &str {
        &self.name
    }

    fn to_param(&self) -> Param {
        Param::new(&self.name, Value::Slider(self.value, self.range.clone()))
    }

    fn update(&mut self, value: Value) {
        if let Value::Slider(val, _) = value {
            self.value = val.clamp(*self.range.start(), *self.range.end());
        }
    }

    fn value(&self) -> Self::Value {
        self.value
    }
}

/// Create a dynamic slider parameter with a range.
///
/// # Example
///
/// ```
/// use snowscape::dynamic;
/// let padding = dynamic::slider("Padding", 0.0..=100.0, 16.0);
/// ```
pub fn slider(name: impl Into<String>, range: RangeInclusive<f32>, default: f32) -> SliderParam {
    SliderParam {
        name: name.into(),
        value: default.clamp(*range.start(), *range.end()),
        range,
    }
}

/// A color parameter that produces `Color` values.
#[derive(Debug, Clone)]
pub struct ColorParam {
    name: String,
    color: Color,
}

impl DynamicParam for ColorParam {
    type Value = Color;

    fn name(&self) -> &str {
        &self.name
    }

    fn to_param(&self) -> Param {
        Param::new(&self.name, Value::Color(self.color))
    }

    fn update(&mut self, value: Value) {
        if let Value::Color(c) = value {
            self.color = c;
        }
    }

    fn value(&self) -> Self::Value {
        self.color
    }
}

/// Create a dynamic color parameter.
///
/// # Example
///
/// ```
/// use snowscape::dynamic;
/// let bg_color = dynamic::color("Background", iced::Color::from_rgb(0.2, 0.4, 0.8));
/// ```
pub fn color(name: impl Into<String>, default: Color) -> ColorParam {
    ColorParam {
        name: name.into(),
        color: default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_param_basic() {
        let param = text("my_text", "hello");
        assert_eq!(param.name(), "my_text");
        assert_eq!(param.value(), "hello");
    }

    #[test]
    fn text_param_update() {
        let mut param = text("my_text", "hello");
        param.update(Value::Text("world".to_string()));
        assert_eq!(param.value(), "world");
    }

    #[test]
    fn number_param_basic() {
        let param = number("my_number", 42);
        assert_eq!(param.name(), "my_number");
        assert_eq!(param.value(), 42);
    }

    #[test]
    fn number_param_update() {
        let mut param = number("my_number", 42);
        param.update(Value::I32(100));
        assert_eq!(param.value(), 100);
    }

    #[test]
    fn bool_param_basic() {
        let param = boolean("my_bool", true);
        assert_eq!(param.name(), "my_bool");
        assert_eq!(param.value(), true);
    }

    #[test]
    fn bool_param_update() {
        let mut param = boolean("my_bool", true);
        param.update(Value::Bool(false));
        assert_eq!(param.value(), false);
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Size {
        Small,
        Medium,
        Large,
    }
    impl std::fmt::Display for Size {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[test]
    fn select_param_basic() {
        let param = select(
            "Size",
            &[Size::Small, Size::Medium, Size::Large],
            Size::Medium,
        );
        assert_eq!(param.name(), "Size");
        assert_eq!(param.value(), Size::Medium);
    }

    #[test]
    fn select_param_update() {
        let mut param = select(
            "Size",
            &[Size::Small, Size::Medium, Size::Large],
            Size::Small,
        );
        param.update(Value::Select(2, vec![]));
        assert_eq!(param.value(), Size::Large);
    }

    #[test]
    fn slider_param_basic() {
        let param = slider("Padding", 0.0..=100.0, 50.0);
        assert_eq!(param.name(), "Padding");
        assert_eq!(param.value(), 50.0);
    }

    #[test]
    fn slider_param_update() {
        let mut param = slider("Padding", 0.0..=100.0, 50.0);
        param.update(Value::Slider(75.0, 0.0..=100.0));
        assert_eq!(param.value(), 75.0);
    }

    /// Slider should clamp values to the defined range.
    #[test]
    fn slider_param_clamps() {
        let mut param = slider("Padding", 0.0..=100.0, 50.0);
        param.update(Value::Slider(150.0, 0.0..=100.0));
        assert_eq!(param.value(), 100.0);
    }

    #[test]
    fn color_param_basic() {
        let red = Color::from_rgb(1.0, 0.0, 0.0);
        let param = color("Background", red);
        assert_eq!(param.name(), "Background");
        let value = param.value();
        assert_eq!(value, red);
    }

    #[test]
    fn color_param_update() {
        let red = Color::from_rgb(1.0, 0.0, 0.0);
        let blue = Color::from_rgb(0.0, 0.0, 1.0);
        let mut param = color("Background", red);
        param.update(Value::Color(blue));
        let value = param.value();
        assert_eq!(value, blue);
    }
}
