//! Event types — device lifecycle, tablet AI, scheduler decisions, health.
//!
//! Published on various `*/up/events/**` topics.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::validate::PayloadValidation;

/// Tablet on-device AI inference result, relayed by nexox.
/// Topic: `nexox/up/events/tablet-ai/<node>/<tablet>`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabletAiResult {
    pub tablet_id: String,
    pub model_id: String,
    pub inference_ms: u32,
    pub result: serde_json::Value,
    pub success: bool,
}

impl PayloadValidation for TabletAiResult {
    fn is_valid(&self) -> bool {
        !self.tablet_id.is_empty() && !self.model_id.is_empty()
    }
}

/// minox scheduler decision event.
/// Topic: `minox/up/events/decision-made`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvent {
    pub task_id: Uuid,
    pub decision: String,
    pub target_node: String,
    pub target_tier: u8,
    pub latency_ms: u32,
}

impl PayloadValidation for DecisionEvent {
    fn is_valid(&self) -> bool {
        !self.target_node.is_empty() && (1..=4).contains(&self.target_tier)
    }
}

/// minox overload prediction event.
/// Topic: `minox/up/events/overload-predicted`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverloadForecast {
    pub node: String,
    pub predicted_at_secs: u32,
    pub confidence: f32,
    pub recommended_action: String,
}

impl PayloadValidation for OverloadForecast {
    fn is_valid(&self) -> bool {
        !self.node.is_empty() && (0.0..=1.0).contains(&self.confidence)
    }
}

/// Service health status, published by nexox supervisor.
/// Topic: `nexox/up/health/<service>/heartbeat`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub service: String,
    pub status: HealthState,
    pub uptime_secs: u64,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Stall,
    Crashed,
    Missing,
    Stopped,
}

impl PayloadValidation for HealthStatus {
    fn is_valid(&self) -> bool {
        !self.service.is_empty()
    }
}

/// Device connected event (ECST pattern — full state in payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConnectedEvent {
    pub serial: String,
    pub model: String,
    pub os: String,
    pub manufacturer: String,
    pub capabilities: Vec<String>,
}

impl PayloadValidation for DeviceConnectedEvent {
    fn is_valid(&self) -> bool {
        !self.serial.is_empty()
    }
}
