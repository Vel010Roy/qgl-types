//! PayloadValidation trait — common payload validation for all QGL projects.
//!
//! Every payload struct implements this trait. Receivers check is_valid()
//! before processing. Invalid payloads are skipped with a warning log
//! and metric increment.

/// Trait for validating Zenoh payload contents.
///
/// Returns false if the payload is empty, malformed, or otherwise invalid.
/// Receivers should skip invalid payloads rather than crashing.
pub trait PayloadValidation {
    fn is_valid(&self) -> bool;
}
