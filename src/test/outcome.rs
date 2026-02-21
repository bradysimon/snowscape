/// The outcome from running a single test.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// The name of the test.
    pub name: String,
    /// Error message if the test failed.
    pub error: Option<String>,
}

impl Outcome {
    /// Returns true if the test passed.
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Creates a passing test outcome with the given name.
    pub fn passed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            error: None,
        }
    }

    /// Creates a failing test outcome with the given name and error message.
    pub fn failed(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            error: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Success being true means there is no error message.
    #[test]
    fn is_success_true() {
        let outcome = Outcome {
            name: "test1".to_string(),
            error: None,
        };
        assert!(outcome.is_success());
        assert!(outcome.error.is_none());
    }

    /// Success being false means there is an error message.
    #[test]
    fn is_success_false() {
        let outcome = Outcome {
            name: "test2".to_string(),
            error: Some("Expected 2 but got 3".to_string()),
        };
        assert!(!outcome.is_success());
        assert!(outcome.error.is_some());
    }

    /// The `passed` fn should create a successful outcome with the given name.
    #[test]
    fn outcome_passed_constructor() {
        let outcome = Outcome::passed("test1");
        assert!(outcome.is_success());
        assert_eq!(outcome.name, "test1");
        assert!(outcome.error.is_none());
    }

    /// The `failed` fn should create a failed outcome with the given name and error message.
    #[test]
    fn outcome_failed_constructor() {
        let outcome = Outcome::failed("test2", "Expected 2 but got 3");
        assert!(!outcome.is_success());
        assert_eq!(outcome.name, "test2");
        assert_eq!(outcome.error.as_deref(), Some("Expected 2 but got 3"));
    }
}
