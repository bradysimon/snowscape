use crate::dynamic::Value;

/// A dynamic parameter that can be adjusted in the configuration pane.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub value: Value,
}

impl Param {
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
pub fn text(name: impl Into<String>, value: impl Into<String>) -> TextParam {
    TextParam {
        name: name.into(),
        value: value.into(),
    }
}

/// Create a dynamic number parameter.
pub fn number(name: impl Into<String>, value: i32) -> NumberParam {
    NumberParam {
        name: name.into(),
        value,
    }
}

/// Create a dynamic boolean parameter.
pub fn boolean(name: impl Into<String>, value: bool) -> BoolParam {
    BoolParam {
        name: name.into(),
        value,
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
}
