use serde::Serialize;

/// A policy violation with a stable machine-readable code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Violation {
    /// Stable diagnostic code.
    pub code: &'static str,
    /// Package associated with the violation.
    pub crate_name: String,
    /// Human-readable explanation.
    pub message: String,
    /// Approved exception suppressing this violation, when present.
    pub exception_id: Option<String>,
}
