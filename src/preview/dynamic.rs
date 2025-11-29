mod extract_params;
pub mod param;
pub mod stateful;
pub mod stateless;

pub use extract_params::ExtractParams;
pub use param::{Param, boolean, number, text};
pub use stateful::stateful;
pub use stateless::stateless;

/// A dynamic parameter value used within [`Param`].
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Text(String),
    I32(i32),
}
