//! Envelope<T> — mandatory wrapper for all Zenoh publish payloads.
//!
//! Every message on the QGL Zenoh bus is wrapped in an Envelope that carries
//! a correlation_id for cross-service tracing, a parent_span_id for causal
//! chains, a timestamp, and the source service identifier.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type alias for correlation ID (cross-service trace).
pub type CorrelationId = Uuid;

/// Type alias for service identifier (e.g. "nexox-controller", "minox-scheduler").
pub type ServiceId = String;

/// Mandatory wrapper for all Zenoh payloads. No bare payloads allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub correlation_id: CorrelationId,
    pub parent_span_id: Option<Uuid>,
    pub emitted_at: DateTime<Utc>,
    pub source_service: ServiceId,
    pub payload: T,
}

impl<T> Envelope<T> {
    /// Create a new Envelope with a fresh correlation_id and current timestamp.
    pub fn new(source_service: impl Into<ServiceId>, payload: T) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            parent_span_id: None,
            emitted_at: Utc::now(),
            source_service: source_service.into(),
            payload,
        }
    }

    /// Create a child envelope that inherits the parent's correlation_id.
    pub fn child(parent: &Envelope<impl std::any::Any>, source_service: impl Into<ServiceId>, payload: T) -> Self {
        Self {
            correlation_id: parent.correlation_id,
            parent_span_id: Some(parent.correlation_id),
            emitted_at: Utc::now(),
            source_service: source_service.into(),
            payload,
        }
    }
}
