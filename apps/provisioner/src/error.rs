use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub type Result<T> = std::result::Result<T, HarnessError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessError {
    message: String,
}

impl HarnessError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn with_context(context: &str, source: impl Display) -> Self {
        Self::new(format!("{context}: {source}"))
    }
}

impl Display for HarnessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for HarnessError {}

impl From<std::io::Error> for HarnessError {
    fn from(value: std::io::Error) -> Self {
        Self::with_context("io error", value)
    }
}

impl From<serde_json::Error> for HarnessError {
    fn from(value: serde_json::Error) -> Self {
        Self::with_context("json error", value)
    }
}

impl From<std::num::ParseIntError> for HarnessError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::with_context("integer parse error", value)
    }
}
