//! qgl-types — Shared payload types for the QGL ecosystem.
//!
//! All Zenoh topic payloads across nexox/minox/argox are defined here.
//! No project-local struct redefinition allowed (MSA principle #2).
//!
//! Every payload is wrapped in `Envelope<T>` (mandatory) and implements
//! `PayloadValidation` (mandatory).

pub mod commands;
pub mod display;
pub mod envelope;
pub mod events;
pub mod input;
pub mod logs;
pub mod metrics;
pub mod state;
pub mod validate;

// Re-export commonly used types at crate root.
pub use envelope::{CorrelationId, Envelope, ServiceId};
pub use validate::PayloadValidation;
