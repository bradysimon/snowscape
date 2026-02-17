//! Size input handling for numeric window dimensions.

/// Keeps track of the display string and parsed value for a size input.
#[derive(Debug, Clone)]
pub struct SizeInput {
    /// The raw display string, sanitized to remove invalid characters.
    display: String,
    /// The parsed numeric value when valid, or `None` when invalid.
    value: Option<f32>,
}

impl SizeInput {
    /// Creates a size input from an initial display value.
    pub(crate) fn new(display: impl Into<String>) -> Self {
        let display = sanitize_size_input(&display.into());
        let value = parse_size_input(&display);
        Self { display, value }
    }

    /// Returns the current display string.
    pub(crate) fn display(&self) -> &str {
        &self.display
    }

    /// Returns the parsed value when valid.
    pub(crate) fn value(&self) -> Option<f32> {
        self.value
    }

    /// Returns true when the display parses to a positive number.
    pub(crate) fn is_valid(&self) -> bool {
        self.value.is_some()
    }

    /// Updates the input from raw user text.
    pub(crate) fn update(&mut self, input: String) {
        let display = sanitize_size_input(&input);
        self.value = parse_size_input(&display);
        self.display = display;
    }
}

/// Removes non-numeric characters while keeping the first dot. Strips negative signs.
fn sanitize_size_input(input: &str) -> String {
    let mut sanitized = String::with_capacity(input.len());
    let mut seen_dot = false;

    for ch in input.chars() {
        if ch.is_ascii_digit() {
            sanitized.push(ch);
        } else if ch == '.' && !seen_dot {
            sanitized.push(ch);
            seen_dot = true;
        }
    }

    sanitized
}

/// Parses a size value and enforces that it is greater than zero.
fn parse_size_input(input: &str) -> Option<f32> {
    if input.trim().is_empty() {
        return None;
    }

    let value = input.parse::<f32>().ok()?;
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_validates_initial_input() {
        let input = SizeInput::new("12.5");
        assert_eq!(input.display(), "12.5");
        assert_eq!(input.value(), Some(12.5));
    }

    #[test]
    fn new_sanitizes_initial_input() {
        let input = SizeInput::new("12ab.3.4");
        assert_eq!(input.display(), "12.34");
        assert_eq!(input.value(), Some(12.34));
    }

    /// The size input should be valid if there is a number present.
    #[test]
    fn is_valid_true() {
        let input = SizeInput {
            display: "12.5".to_string(),
            value: Some(12.5),
        };
        assert!(input.is_valid());
    }

    /// Entering invalid text should result in an invalid state.
    #[test]
    fn is_valid_false() {
        let input = SizeInput {
            display: "abc".to_string(),
            value: None,
        };
        assert!(!input.is_valid());
    }

    /// Sanitizes by removing non-numeric characters and extra dots.
    #[test]
    fn sanitize_filters_non_numeric() {
        assert_eq!(sanitize_size_input("12ab.3.4"), "12.34");
        assert_eq!(sanitize_size_input("abc"), "");
    }

    /// Parses only positive values and rejects empty inputs.
    #[test]
    fn parse_rejects_empty_or_zero() {
        assert_eq!(parse_size_input(""), None);
        assert_eq!(parse_size_input("0"), None);
        assert_eq!(parse_size_input("0.0"), None);
        assert_eq!(parse_size_input("0.5"), Some(0.5));
    }

    /// Tracks the display and parsed value on updates.
    #[test]
    fn update_tracks_display_and_value() {
        let mut input = SizeInput::new("12a");
        assert_eq!(input.display(), "12");
        assert_eq!(input.value(), Some(12.0));

        input.update("abc".to_string());
        assert_eq!(input.display(), "");
        assert_eq!(input.value(), None);
    }
}
